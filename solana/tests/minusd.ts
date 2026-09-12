import * as anchor from "@coral-xyz/anchor";
import { BN, Program } from "@coral-xyz/anchor";
import {
  TOKEN_PROGRAM_ID,
  createTransferInstruction,
  getAccount,
  getAssociatedTokenAddressSync,
  getMint,
  getOrCreateAssociatedTokenAccount,
} from "@solana/spl-token";
import { Keypair, PublicKey, SystemProgram, Transaction } from "@solana/web3.js";
import { expect } from "chai";
import { Minusd } from "../target/types/minusd";

const UNIT = 1_000_000;
const TOKEN_PROGRAM = TOKEN_PROGRAM_ID;

describe("minusd lifecycle", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.Minusd as Program<Minusd>;
  const connection = provider.connection;
  const payer = (provider.wallet as anchor.Wallet).payer;
  const admin = provider.wallet.publicKey;

  const pauser = Keypair.generate();
  const compliance = Keypair.generate();
  const alice = Keypair.generate();
  const bob = Keypair.generate();
  const stranger = Keypair.generate();

  const [configPda] = PublicKey.findProgramAddressSync(
    [Buffer.from("config")],
    program.programId
  );
  const [vaultAuthority] = PublicKey.findProgramAddressSync(
    [Buffer.from("vault_authority")],
    program.programId
  );
  const [vault] = PublicKey.findProgramAddressSync(
    [Buffer.from("vault")],
    program.programId
  );
  const [mockUsdcMint] = PublicKey.findProgramAddressSync(
    [Buffer.from("mock_usdc_mint")],
    program.programId
  );
  const [minusdMint] = PublicKey.findProgramAddressSync(
    [Buffer.from("minusd_mint")],
    program.programId
  );

  let aliceUsdc: PublicKey;
  let aliceMinusd: PublicKey;
  let bobUsdc: PublicKey;
  let bobMinusd: PublicKey;

  function frozenPda(owner: PublicKey): PublicKey {
    return PublicKey.findProgramAddressSync(
      [Buffer.from("frozen"), owner.toBuffer()],
      program.programId
    )[0];
  }

  async function airdrop(pk: PublicKey, sol = 2): Promise<void> {
    const sig = await connection.requestAirdrop(pk, sol * anchor.web3.LAMPORTS_PER_SOL);
    const latest = await connection.getLatestBlockhash();
    await connection.confirmTransaction({ signature: sig, ...latest }, "confirmed");
  }

  async function tokenBalance(ata: PublicKey): Promise<number> {
    return Number((await getAccount(connection, ata)).amount);
  }

  async function mintSupply(mint: PublicKey): Promise<number> {
    return Number((await getMint(connection, mint)).supply);
  }

  async function assertPeg(): Promise<void> {
    const vaultBal = await tokenBalance(vault);
    const supply = await mintSupply(minusdMint);
    expect(vaultBal, "vault MockUSDC != MINUSD supply").to.equal(supply);
  }

  async function faucet(owner: Keypair, ata: PublicKey, amount: number): Promise<void> {
    await program.methods
      .mintMockUsdc(new BN(amount))
      .accounts({
        config: configPda,
        mockUsdcMint,
        recipientUsdc: ata,
        vaultAuthority,
        tokenProgram: TOKEN_PROGRAM,
      })
      .rpc();
  }

  async function acquire(caller: Keypair, callerUsdcAta: PublicKey, recipientAta: PublicKey, amount: number) {
    const recipientOwner = (await getAccount(connection, recipientAta)).owner;
    return program.methods
      .acquire(new BN(amount))
      .accounts({
        caller: caller.publicKey,
        config: configPda,
        mockUsdcMint,
        minusdMint,
        callerUsdc: callerUsdcAta,
        vault,
        recipientMinusd: recipientAta,
        vaultAuthority,
        callerFreeze: frozenPda(caller.publicKey),
        recipientFreeze: frozenPda(recipientOwner),
        tokenProgram: TOKEN_PROGRAM,
      })
      .signers([caller])
      .rpc();
  }

  async function redeem(caller: Keypair, callerMinusdAta: PublicKey, recipientUsdcAta: PublicKey, amount: number) {
    const recipientOwner = (await getAccount(connection, recipientUsdcAta)).owner;
    return program.methods
      .redeem(new BN(amount))
      .accounts({
        caller: caller.publicKey,
        config: configPda,
        mockUsdcMint,
        minusdMint,
        callerMinusd: callerMinusdAta,
        vault,
        recipientUsdc: recipientUsdcAta,
        vaultAuthority,
        callerFreeze: frozenPda(caller.publicKey),
        recipientFreeze: frozenPda(recipientOwner),
        tokenProgram: TOKEN_PROGRAM,
      })
      .signers([caller])
      .rpc();
  }

  async function transferMinusd(from: Keypair, fromAta: PublicKey, toAta: PublicKey, amount: number) {
    const tx = new Transaction().add(
      createTransferInstruction(fromAta, toAta, from.publicKey, amount)
    );
    return provider.sendAndConfirm(tx, [from]);
  }

  async function freezeOwner(owner: PublicKey, minusdAta: PublicKey) {
    return program.methods
      .freeze()
      .accounts({
        compliance: compliance.publicKey,
        config: configPda,
        owner,
        frozenOwner: frozenPda(owner),
        minusdAccount: minusdAta,
        minusdMint,
        vaultAuthority,
        tokenProgram: TOKEN_PROGRAM,
        systemProgram: SystemProgram.programId,
      })
      .signers([compliance])
      .rpc();
  }

  async function unfreezeOwner(owner: PublicKey, minusdAta: PublicKey) {
    return program.methods
      .unfreeze()
      .accounts({
        compliance: compliance.publicKey,
        config: configPda,
        owner,
        frozenOwner: frozenPda(owner),
        minusdAccount: minusdAta,
        minusdMint,
        vaultAuthority,
        tokenProgram: TOKEN_PROGRAM,
        systemProgram: SystemProgram.programId,
      })
      .signers([compliance])
      .rpc();
  }

  function expectAnchorCode(err: unknown, code: string) {
    const e = err as { error?: { errorCode?: { code?: string } }; message?: string };
    const got = e.error?.errorCode?.code;
    expect(got, e.message ?? String(err)).to.equal(code);
  }

  before(async () => {
    await Promise.all([
      airdrop(pauser.publicKey),
      airdrop(compliance.publicKey),
      airdrop(alice.publicKey),
      airdrop(bob.publicKey),
      airdrop(stranger.publicKey),
    ]);

    await program.methods
      .initialize(admin, pauser.publicKey, compliance.publicKey)
      .accounts({
        payer: admin,
        config: configPda,
        vaultAuthority,
        mockUsdcMint,
        minusdMint,
        vault,
        tokenProgram: TOKEN_PROGRAM,
        systemProgram: SystemProgram.programId,
      })
      .rpc();

    aliceUsdc = (await getOrCreateAssociatedTokenAccount(connection, payer, mockUsdcMint, alice.publicKey)).address;
    aliceMinusd = (await getOrCreateAssociatedTokenAccount(connection, payer, minusdMint, alice.publicKey)).address;
    bobUsdc = (await getOrCreateAssociatedTokenAccount(connection, payer, mockUsdcMint, bob.publicKey)).address;
    bobMinusd = (await getOrCreateAssociatedTokenAccount(connection, payer, minusdMint, bob.publicKey)).address;

    await faucet(alice, aliceUsdc, 1_000 * UNIT);
  });

  it("decimals are 6 on both mints", async () => {
    const usdc = await getMint(connection, mockUsdcMint);
    const minusd = await getMint(connection, minusdMint);
    expect(usdc.decimals).to.equal(6);
    expect(minusd.decimals).to.equal(6);
    expect(Number(usdc.supply)).to.equal(1_000 * UNIT);
    expect(Number(minusd.supply)).to.equal(0);
  });

  it("acquire, SPL transfer, and redeem keep the peg", async () => {
    const acquireAmount = 100 * UNIT;
    await acquire(alice, aliceUsdc, aliceMinusd, acquireAmount);

    expect(await tokenBalance(aliceMinusd)).to.equal(acquireAmount);
    expect(await tokenBalance(aliceUsdc)).to.equal(900 * UNIT);
    expect(await tokenBalance(vault)).to.equal(acquireAmount);
    await assertPeg();

    await transferMinusd(alice, aliceMinusd, bobMinusd, 40 * UNIT);
    expect(await tokenBalance(aliceMinusd)).to.equal(60 * UNIT);
    expect(await tokenBalance(bobMinusd)).to.equal(40 * UNIT);
    expect(await mintSupply(minusdMint)).to.equal(acquireAmount);
    expect(await tokenBalance(vault)).to.equal(acquireAmount);

    const redeemAmount = 25 * UNIT;
    await redeem(alice, aliceMinusd, aliceUsdc, redeemAmount);

    expect(await tokenBalance(aliceMinusd)).to.equal(35 * UNIT);
    expect(await tokenBalance(aliceUsdc)).to.equal(925 * UNIT);
    expect(await mintSupply(minusdMint)).to.equal(75 * UNIT);
    await assertPeg();
  });

  it("unauthorized admin, pause, and freeze fail", async () => {
    try {
      await program.methods
        .setPauser(stranger.publicKey)
        .accounts({
          admin: stranger.publicKey,
          config: configPda,
        })
        .signers([stranger])
        .rpc();
      expect.fail("stranger setPauser should fail");
    } catch (err) {
      // has_one = admin constraint, not a custom Unauthorized
      expect(String(err)).to.match(/has_one|ConstraintHasOne|2001/i);
    }

    try {
      await program.methods
        .pause()
        .accounts({
          pauser: stranger.publicKey,
          config: configPda,
        })
        .signers([stranger])
        .rpc();
      expect.fail("stranger pause should fail");
    } catch (err) {
      expect(String(err)).to.match(/has_one|ConstraintHasOne|2001/i);
    }

    try {
      await program.methods
        .freeze()
        .accounts({
          compliance: stranger.publicKey,
          config: configPda,
          owner: alice.publicKey,
          frozenOwner: frozenPda(alice.publicKey),
          minusdAccount: aliceMinusd,
          minusdMint,
          vaultAuthority,
          tokenProgram: TOKEN_PROGRAM,
          systemProgram: SystemProgram.programId,
        })
        .signers([stranger])
        .rpc();
      expect.fail("stranger freeze should fail");
    } catch (err) {
      expect(String(err)).to.match(/has_one|ConstraintHasOne|2001/i);
    }
  });

  it("pause blocks acquire and redeem but not SPL transfers", async () => {
    await acquire(alice, aliceUsdc, aliceMinusd, 50 * UNIT);

    await program.methods
      .pause()
      .accounts({ pauser: pauser.publicKey, config: configPda })
      .signers([pauser])
      .rpc();

    try {
      await acquire(alice, aliceUsdc, aliceMinusd, 10 * UNIT);
      expect.fail("paused acquire should fail");
    } catch (err) {
      expectAnchorCode(err, "Paused");
    }

    try {
      await redeem(alice, aliceMinusd, aliceUsdc, 10 * UNIT);
      expect.fail("paused redeem should fail");
    } catch (err) {
      expectAnchorCode(err, "Paused");
    }

    const bobBefore = await tokenBalance(bobMinusd);
    await transferMinusd(alice, aliceMinusd, bobMinusd, 10 * UNIT);
    expect(await tokenBalance(bobMinusd)).to.equal(bobBefore + 10 * UNIT);

    await program.methods
      .unpause()
      .accounts({ pauser: pauser.publicKey, config: configPda })
      .signers([pauser])
      .rpc();

    const supplyBefore = await mintSupply(minusdMint);
    await acquire(alice, aliceUsdc, aliceMinusd, 5 * UNIT);
    expect(await mintSupply(minusdMint)).to.equal(supplyBefore + 5 * UNIT);
    await assertPeg();
  });

  it("frozen sender and recipient fail acquire, transfer, and redeem", async () => {
    await acquire(alice, aliceUsdc, aliceMinusd, 80 * UNIT);
    await freezeOwner(alice.publicKey, aliceMinusd);

    try {
      await acquire(alice, aliceUsdc, bobMinusd, 10 * UNIT);
      expect.fail("frozen caller acquire should fail");
    } catch (err) {
      expectAnchorCode(err, "AccountIsFrozen");
    }

    try {
      await transferMinusd(alice, aliceMinusd, bobMinusd, UNIT);
      expect.fail("frozen sender transfer should fail");
    } catch (err) {
      expect(String(err)).to.match(/frozen|0x11|17/i);
    }

    try {
      await redeem(alice, aliceMinusd, aliceUsdc, UNIT);
      expect.fail("frozen caller redeem should fail");
    } catch (err) {
      expectAnchorCode(err, "AccountIsFrozen");
    }

    await unfreezeOwner(alice.publicKey, aliceMinusd);
    await acquire(alice, aliceUsdc, aliceMinusd, 10 * UNIT);

    await freezeOwner(bob.publicKey, bobMinusd);

    try {
      await transferMinusd(alice, aliceMinusd, bobMinusd, UNIT);
      expect.fail("frozen recipient transfer should fail");
    } catch (err) {
      expect(String(err)).to.match(/frozen|0x11|17/i);
    }

    try {
      await acquire(alice, aliceUsdc, bobMinusd, 10 * UNIT);
      expect.fail("frozen recipient acquire should fail");
    } catch (err) {
      expectAnchorCode(err, "AccountIsFrozen");
    }

    try {
      await redeem(alice, aliceMinusd, bobUsdc, UNIT);
      expect.fail("frozen recipient redeem should fail");
    } catch (err) {
      expectAnchorCode(err, "AccountIsFrozen");
    }

    await unfreezeOwner(bob.publicKey, bobMinusd);
  });

  it("insufficient MINUSD and MockUSDC fail without changing balances", async () => {
    const aliceMin = await tokenBalance(aliceMinusd);
    const aliceU = await tokenBalance(aliceUsdc);
    const supply = await mintSupply(minusdMint);
    const vaultBal = await tokenBalance(vault);

    try {
      await redeem(alice, aliceMinusd, aliceUsdc, aliceMin + UNIT);
      expect.fail("over-redeem should fail");
    } catch (err) {
      expect(String(err)).to.match(/insufficient|0x1\b/i);
    }

    try {
      await acquire(alice, aliceUsdc, aliceMinusd, aliceU + UNIT);
      expect.fail("over-acquire should fail");
    } catch (err) {
      expect(String(err)).to.match(/insufficient|0x1\b/i);
    }

    expect(await tokenBalance(aliceMinusd)).to.equal(aliceMin);
    expect(await tokenBalance(aliceUsdc)).to.equal(aliceU);
    expect(await mintSupply(minusdMint)).to.equal(supply);
    expect(await tokenBalance(vault)).to.equal(vaultBal);
    await assertPeg();
  });

  it("zero amount and invalid accounts fail", async () => {
    try {
      await acquire(alice, aliceUsdc, aliceMinusd, 0);
      expect.fail("zero acquire should fail");
    } catch (err) {
      expectAnchorCode(err, "ZeroAmount");
    }

    try {
      await redeem(alice, aliceMinusd, aliceUsdc, 0);
      expect.fail("zero redeem should fail");
    } catch (err) {
      expectAnchorCode(err, "ZeroAmount");
    }

    try {
      await faucet(alice, aliceUsdc, 0);
      expect.fail("zero faucet should fail");
    } catch (err) {
      expectAnchorCode(err, "ZeroAmount");
    }

    // Wrong mint: MockUSDC ATA cannot be a MINUSD recipient.
    try {
      await acquire(alice, aliceUsdc, aliceUsdc, UNIT);
      expect.fail("wrong-mint recipient should fail");
    } catch (err) {
      expect(String(err)).to.match(/constraint|mint|201/i);
    }
  });

  it("1e6 is one whole unit, not 1e9", async () => {
    const before = await tokenBalance(aliceMinusd);
    await acquire(alice, aliceUsdc, aliceMinusd, 1 * UNIT);
    expect(await tokenBalance(aliceMinusd)).to.equal(before + 1_000_000);
    expect(await tokenBalance(aliceMinusd)).to.not.equal(before + 1_000_000_000);
    await assertPeg();
  });

  it("peg holds after a mixed acquire / transfer / redeem sequence", async () => {
    // Isolate by redeeming alice+bob down if needed is hard; snapshot and apply a known delta.
    const supply0 = await mintSupply(minusdMint);
    const vault0 = await tokenBalance(vault);
    const aliceMin0 = await tokenBalance(aliceMinusd);
    const bobMin0 = await tokenBalance(bobMinusd);

    await acquire(alice, aliceUsdc, aliceMinusd, 100 * UNIT);
    await transferMinusd(alice, aliceMinusd, bobMinusd, 40 * UNIT);
    await acquire(alice, aliceUsdc, aliceMinusd, 25 * UNIT);
    await redeem(alice, aliceMinusd, aliceUsdc, 30 * UNIT);
    await transferMinusd(bob, bobMinusd, aliceMinusd, 10 * UNIT);
    await redeem(alice, aliceMinusd, bobUsdc, 20 * UNIT);

    expect(await mintSupply(minusdMint)).to.equal(supply0 + 75 * UNIT);
    expect(await tokenBalance(vault)).to.equal(vault0 + 75 * UNIT);
    expect(await tokenBalance(aliceMinusd)).to.equal(aliceMin0 + 45 * UNIT);
    expect(await tokenBalance(bobMinusd)).to.equal(bobMin0 + 30 * UNIT);
    await assertPeg();
  });

  it("a failed instruction does not change balances", async () => {
    await acquire(alice, aliceUsdc, aliceMinusd, 20 * UNIT);

    const snap = {
      aliceUsdc: await tokenBalance(aliceUsdc),
      aliceMin: await tokenBalance(aliceMinusd),
      bobMin: await tokenBalance(bobMinusd),
      supply: await mintSupply(minusdMint),
      vault: await tokenBalance(vault),
    };

    try {
      await acquire(alice, aliceUsdc, aliceMinusd, snap.aliceUsdc + UNIT);
      expect.fail("should fail");
    } catch {
      /* expected */
    }

    try {
      await redeem(alice, aliceMinusd, aliceUsdc, snap.aliceMin + UNIT);
      expect.fail("should fail");
    } catch {
      /* expected */
    }

    try {
      await transferMinusd(alice, aliceMinusd, bobMinusd, snap.aliceMin + UNIT);
      expect.fail("should fail");
    } catch {
      /* expected */
    }

    await freezeOwner(bob.publicKey, bobMinusd);
    try {
      await transferMinusd(alice, aliceMinusd, bobMinusd, UNIT);
      expect.fail("should fail");
    } catch {
      /* expected */
    }
    await unfreezeOwner(bob.publicKey, bobMinusd);

    expect(await tokenBalance(aliceUsdc)).to.equal(snap.aliceUsdc);
    expect(await tokenBalance(aliceMinusd)).to.equal(snap.aliceMin);
    expect(await tokenBalance(bobMinusd)).to.equal(snap.bobMin);
    expect(await mintSupply(minusdMint)).to.equal(snap.supply);
    expect(await tokenBalance(vault)).to.equal(snap.vault);
  });
});
