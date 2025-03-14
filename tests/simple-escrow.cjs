const anchor = require("@coral-xyz/anchor");
const { Program } = anchor;
const { SimpleEscrow } = require("../target/types/simple_escrow");
const { Keypair, PublicKey, LAMPORTS_PER_SOL } = require("@solana/web3.js");

describe("simple-escrow", () => {
  anchor.setProvider(anchor.AnchorProvider.env());
  const provider = anchor.getProvider();
  const program = anchor.workspace.SimpleEscrow;
  const seller = Keypair.generate();

  it("Creates an escrow", async () => {
    const escrowId = new anchor.BN(1);
    const amount = new anchor.BN(1000000);

    const [escrowPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("escrow"), escrowId.toArrayLike(Buffer, "le", 8)],
      program.programId
    );

    const signature = await provider.connection.requestAirdrop(
      seller.publicKey,
      2 * LAMPORTS_PER_SOL
    );
    await provider.connection.confirmTransaction(signature);

    try {
      const tx = await program.methods
        .createEscrow(escrowId, amount)
        .accounts({
          seller: seller.publicKey,
          escrow: escrowPda,
          systemProgram: anchor.web3.SystemProgram.programId,
        })
        .signers([seller])
        .rpc();

      console.log("Transaction signature:", tx);
      console.log("Escrow PDA:", escrowPda.toBase58());

      const escrowAccount = await program.account.escrow.fetch(escrowPda);
      console.log("Escrow account:", escrowAccount);
    } catch (error) {
      console.error("Test failed with error:", error);
      if (error.logs) {
        console.error("Transaction logs:", error.logs);
      }
      throw error;
    }
  });
});