import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { AnchorAmm } from "../target/types/anchor_amm";
import { Account, ASSOCIATED_TOKEN_PROGRAM_ID, createMint, getAccount, getAssociatedTokenAddress, getOrCreateAssociatedTokenAccount, mintTo, TOKEN_PROGRAM_ID } from "@solana/spl-token";
import { Keypair, LAMPORTS_PER_SOL, MessageAccountKeys, PublicKey } from "@solana/web3.js";

describe("anchor-amm", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());
  let provider = anchor.AnchorProvider.env();
  const program = anchor.workspace.anchorAmm as Program<AnchorAmm>;
  console.log("progrm id: " , program.programId );
  let seed = new anchor.BN(1);
  let mint_x:anchor.web3.PublicKey;
  let mint_y:anchor.web3.PublicKey;
  let mint_lp:anchor.web3.PublicKey;
  let user = Keypair.generate();
  let vault_x:anchor.web3.PublicKey;
  let vault_y:anchor.web3.PublicKey;
  let user_ata_lp:anchor.web3.PublicKey;
  let user_ata_x:Account;
  let user_ata_y:Account;
  let config = anchor.web3.PublicKey.findProgramAddressSync([Buffer.from("config") , Buffer.from(seed.toArray("le" , 8))] , program.programId)[0];
  mint_lp = anchor.web3.PublicKey.findProgramAddressSync([Buffer.from("lp") , config.toBytes()] , program.programId)[0];


  //we need to send some sol to use{r and create a mint to mint_x and mint_y
  before(async () => {
    //sending sol to user
    let sendSol = async (user:PublicKey) => {
      let tx = new anchor.web3.Transaction().add(
        anchor.web3.SystemProgram.transfer({
          fromPubkey:provider.wallet.publicKey,
          toPubkey:user,
          lamports: 5 * LAMPORTS_PER_SOL
        })
      )

      let sig = await provider.sendAndConfirm(tx);
      console.log("sol sent to user: ",sig);
    }
    await sendSol(user.publicKey);

    //creating mint
    mint_x = await createMint(provider.connection , provider.wallet.payer , provider.wallet.publicKey , null , 6 );
    mint_y = await createMint(provider.connection , provider.wallet.payer , provider.wallet.publicKey , null , 6 );
    console.log("mint x created:", mint_x.toBase58());
    console.log("mint y created:", mint_y.toBase58());

    vault_x = await getAssociatedTokenAddress(mint_x , config , true);
    vault_y = await getAssociatedTokenAddress(mint_y , config, true);
    console.log("vault x created: ", vault_x.toBase58());
    console.log("vault y created: ", vault_y.toBase58());

    user_ata_x = await getOrCreateAssociatedTokenAccount(provider.connection , user , mint_x , user.publicKey , true);
    user_ata_y = await getOrCreateAssociatedTokenAccount(provider.connection , user , mint_y , user.publicKey , true);
    await mintTo(provider.connection , provider.wallet.payer , mint_x , user_ata_x.address , provider.wallet.payer , 1_0_000_00);
    await mintTo(provider.connection , provider.wallet.payer , mint_y , user_ata_y.address , provider.wallet.payer , 1_0_000_00);
    user_ata_lp = await getAssociatedTokenAddress(mint_lp , user.publicKey, true);

  });


  //creating pool
  it("initializing pool" , async () => {
    let tx  = await program.methods
          .initialize(seed , provider.wallet.publicKey , 300)
          .accountsPartial({
            initializer:provider.wallet.publicKey,
            mintX:mint_x,
            mintY:mint_y,
            config:config,
            vaultX:vault_x,
            vaultY:vault_y,
            mintLp:mint_lp,
            systemProgram:anchor.web3.SystemProgram.programId,
            tokenProgram:TOKEN_PROGRAM_ID,
            associatedTokenProgram:ASSOCIATED_TOKEN_PROGRAM_ID,
          })
          .signers([provider.wallet.payer])
          .rpc();
    console.log("pool created succefully✅: ",tx);
    let vaultXInfo = await getAccount(provider.connection , vault_x);
    let vaultYInfo = await getAccount(provider.connection , vault_y);
    let userAtaXInfo = await getAccount(provider.connection , user_ata_x.address);
    let userAtaYInfo = await getAccount(provider.connection , user_ata_y.address);
    console.log("vault x :" , vaultXInfo.amount);
    console.log("vault y :" , vaultYInfo.amount);
    console.log("user ata x balance" , userAtaXInfo.amount);
    console.log("user ata y balance" , userAtaYInfo.amount);
  })


  let mintLpTokenAmount = new anchor.BN(2_000);
  let max_x = new anchor.BN(1_000_000);
  let max_y= new anchor.BN(1_000_000);


  //depositing to pool
  it("deposit token to pool" , async () => {

    let tx = await program.methods
          .deposit(mintLpTokenAmount , max_x, max_y)
          .accountsPartial({
            user:user.publicKey,
            mintX:mint_x,
            mintY:mint_y,
            config:config,
            vaultX:vault_x,
            vaultY:vault_y,
            mintLp:mint_lp,
            userLp:user_ata_lp,
            userX:user_ata_x.address,
            userY:user_ata_y.address,
            systemProgram:anchor.web3.SystemProgram.programId,
            tokenProgram:TOKEN_PROGRAM_ID,
            associatedTokenProgram:ASSOCIATED_TOKEN_PROGRAM_ID,
          })
          .signers([user])
          .rpc();
    console.log("token deposited to pool succefully✅:",tx);
    // const sig = await provider.connection.getParsedTransaction(tx, {
    //   commitment: "confirmed",
    //   maxSupportedTransactionVersion: 0,
    // });
    // console.log(sig?.meta?.logMessages?.join("\n"));
    let vaultXInfo = await getAccount(provider.connection , vault_x);
    let vaultYInfo = await getAccount(provider.connection , vault_y);
    let userAtaXInfo = await getAccount(provider.connection , user_ata_x.address);
    let userAtaYInfo = await getAccount(provider.connection , user_ata_y.address);
    let userLpInfo = await getAccount(provider.connection , user_ata_lp);
    console.log("vault x :" , vaultXInfo.amount);
    console.log("vault y :" , vaultYInfo.amount);
    console.log("user ata x balance" , userAtaXInfo.amount);
    console.log("user ata y balance" , userAtaYInfo.amount);
    console.log("user ata lp balance" , userLpInfo.amount);
    let after = await getAccount(provider.connection, user_ata_lp);
console.log("LP after:", after.amount);

  })

  //withdraw token from the pool
  it("withdraw token from the pool" , async () => {
    let tx = await program.methods
          .withdraw(new anchor.BN(1), new anchor.BN(1), new anchor.BN(1))
          .accountsPartial({
            user:user.publicKey,
            mintX:mint_x,
            mintY:mint_y,
            config:config,
            vaultX:vault_x,
            vaultY:vault_y,
            userY:user_ata_y.address,
            userX:user_ata_x.address,
            mintLp:mint_lp,
            userLp:user_ata_lp,
            systemProgram:anchor.web3.SystemProgram.programId,
            tokenProgram:TOKEN_PROGRAM_ID,
            associatedTokenProgram:ASSOCIATED_TOKEN_PROGRAM_ID,
          })
          .signers([user])
          .rpc();
    console.log("token withdraw from pool succefully✅:",tx);
    let vaultXInfo = await getAccount(provider.connection , vault_x);
    let vaultYInfo = await getAccount(provider.connection , vault_y);
    let userAtaXInfo = await getAccount(provider.connection , user_ata_x.address);
    let userAtaYInfo = await getAccount(provider.connection , user_ata_y.address);
    let userLpInfo = await getAccount(provider.connection , user_ata_lp);
    console.log("vault x :" , vaultXInfo.amount);
    console.log("vault y :" , vaultYInfo.amount);
    console.log("user ata x balance" , userAtaXInfo.amount);
    console.log("user ata y balance" , userAtaYInfo.amount);
    console.log("user ata lp balance" , userLpInfo.amount);
  });

});

