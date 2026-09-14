import { describe, expect, it } from "vitest";
import {
  Address,
  Keypair,
  nativeToScVal,
  Networks,
  scValToNative,
  StrKey,
  xdr,
} from "@stellar/stellar-sdk";
import { EscrowClient, EscrowContractError, mapSimulationError } from "./client.js";
import { decodeEscrow, decodeMilestone, milestoneInputsVal } from "./encoding.js";

const CONTRACT = StrKey.encodeContract(Buffer.alloc(32, 3));
const ASSET = StrKey.encodeContract(Buffer.alloc(32, 4));
const buyer = Keypair.fromRawEd25519Seed(Buffer.alloc(32, 1)).publicKey();
const seller = Keypair.fromRawEd25519Seed(Buffer.alloc(32, 2)).publicKey();
const arbitrator = Keypair.fromRawEd25519Seed(Buffer.alloc(32, 5)).publicKey();

function invokeArgs(op: xdr.Operation) {
  const body = op.body;
  if (body.type !== "invokeHostFunction") throw new Error("not an invokeHostFunction op");
  const host = body.invokeHostFunctionOp.hostFunction;
  if (host.type !== "hostFunctionTypeInvokeContract") throw new Error("not a contract invocation");
  return host.invokeContract;
}

function invocation(op: xdr.Operation) {
  const fn = invokeArgs(op);
  return {
    name: fn.functionName.toString(),
    args: fn.args.map((a) => scValToNative(a) as unknown),
  };
}

describe("EscrowClient invocation builders", () => {
  const client = new EscrowClient({ contractId: CONTRACT, networkPassphrase: Networks.TESTNET });

  it("encodes create with milestones and expiration", () => {
    const op = client.createOp({
      buyer,
      seller,
      arbitrator,
      asset: ASSET,
      milestones: [
        { amount: 400n, description: "design" },
        { amount: 600n, description: "build" },
      ],
      expiration: 1_800_000_000,
    });
    const { name, args } = invocation(op);
    expect(name).toBe("create");
    expect(args[0]).toBe(buyer);
    expect(args[3]).toBe(ASSET);
    expect(args[4]).toEqual([
      { amount: 400n, description: "design" },
      { amount: 600n, description: "build" },
    ]);
    expect(args[5]).toBe(1_800_000_000n);
  });

  it("validates milestone counts and amounts client-side", () => {
    expect(() =>
      client.createOp({ buyer, seller, arbitrator, asset: ASSET, milestones: [], expiration: 1 }),
    ).toThrow(RangeError);
    expect(() =>
      client.createOp({
        buyer,
        seller,
        arbitrator,
        asset: ASSET,
        milestones: [{ amount: 0n, description: "x" }],
        expiration: 1,
      }),
    ).toThrow(RangeError);
    expect(() => client.resolveDisputeOp(1, 10_001)).toThrow(RangeError);
  });

  it("encodes the remaining operations", () => {
    expect(invocation(client.fundOp(7n))).toEqual({ name: "fund", args: [7n] });
    expect(invocation(client.cancelOp(7n, seller))).toEqual({ name: "cancel", args: [7n, seller] });
    expect(invocation(client.approveMilestoneOp(7n, 1))).toEqual({
      name: "approve_milestone",
      args: [7n, 1],
    });
    expect(invocation(client.releaseOp(7n, 1, seller))).toEqual({
      name: "release",
      args: [7n, 1, seller],
    });
    expect(invocation(client.refundOp(7n))).toEqual({ name: "refund", args: [7n] });
    expect(invocation(client.openDisputeOp(7n, buyer, "late"))).toEqual({
      name: "open_dispute",
      args: [7n, buyer, "late"],
    });
    expect(invocation(client.resolveDisputeOp(7n, 2_500))).toEqual({
      name: "resolve_dispute",
      args: [7n, 2_500],
    });
  });

  it("targets the configured contract", () => {
    const op = client.fundOp(1);
    const addr = invokeArgs(op).contractAddress;
    expect(Address.fromScAddress(addr).toString()).toBe(CONTRACT);
  });
});

describe("decoders", () => {
  it("decodes an Escrow struct map", () => {
    const entries: [string, xdr.ScVal][] = [
      ["amount", nativeToScVal(1000n, { type: "i128" })],
      ["arbitrator", new Address(arbitrator).toScVal()],
      ["asset", new Address(ASSET).toScVal()],
      ["buyer", new Address(buyer).toScVal()],
      ["created_at", nativeToScVal(1_000_000n, { type: "u64" })],
      ["expiration", nativeToScVal(1_002_592_000n, { type: "u64" })],
      ["id", nativeToScVal(0n, { type: "u64" })],
      ["milestone_count", nativeToScVal(2, { type: "u32" })],
      ["refunded", nativeToScVal(0n, { type: "i128" })],
      ["released", nativeToScVal(400n, { type: "i128" })],
      ["seller", new Address(seller).toScVal()],
      ["status", xdr.ScVal.scvVec([xdr.ScVal.scvSymbol("Funded")])],
    ];
    const val = xdr.ScVal.scvMap(
      entries.map(([k, v]) => new xdr.ScMapEntry({ key: xdr.ScVal.scvSymbol(k), val: v })),
    );
    expect(decodeEscrow(val)).toEqual({
      id: 0n,
      buyer,
      seller,
      arbitrator,
      asset: ASSET,
      amount: 1000n,
      released: 400n,
      refunded: 0n,
      expiration: 1_002_592_000n,
      milestoneCount: 2,
      status: "Funded",
      createdAt: 1_000_000n,
    });
  });

  it("round-trips milestone inputs through the encoder and decoder", () => {
    const vec = milestoneInputsVal([{ amount: 5n, description: "a" }]);
    if (vec.type !== "scvVec" || !vec.vec) throw new Error("expected vec");
    const first = vec.vec[0]!;
    if (first.type !== "scvMap" || !first.map) throw new Error("expected map");
    const withStatus = xdr.ScVal.scvMap([
      ...first.map,
      new xdr.ScMapEntry({ key: xdr.ScVal.scvSymbol("index"), val: xdr.ScVal.scvU32(0) }),
      new xdr.ScMapEntry({
        key: xdr.ScVal.scvSymbol("status"),
        val: xdr.ScVal.scvVec([xdr.ScVal.scvSymbol("Pending")]),
      }),
    ]);
    expect(decodeMilestone(withStatus)).toEqual({
      index: 0,
      amount: 5n,
      description: "a",
      status: "Pending",
    });
  });
});

describe("errors", () => {
  it("maps contract error codes from simulation messages", () => {
    const err = mapSimulationError("HostError: Error(Contract, #10)\n...");
    expect(err).toBeInstanceOf(EscrowContractError);
    expect((err as EscrowContractError).code).toBe(10);
    expect((err as EscrowContractError).name_).toBe("NotExpired");
    expect(mapSimulationError("something else").message).toBe("something else");
  });
});
