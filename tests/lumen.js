const assert = require("assert");
const anchor = require('@project-serum/anchor');
const {SystemProgram} = anchor.web3;
const {
  ASSOCIATED_TOKEN_PROGRAM_ID,
  TOKEN_PROGRAM_ID,
  Token,
} = require("@solana/spl-token");
const {
  sleep,
  getTokenAccount,
  createMint,
  createTokenAccount,
} = require("./utils");
const { token } = require("@project-serum/anchor/dist/cjs/utils");

describe('seer', () => {
  const programName = "seer"
  const provider = anchor.Provider.env();
  // Configure the client to use the local cluster.
  anchor.setProvider(provider);
  const program = anchor.workspace.Lumen;

  const redeemableAmount = new anchor.BN(5000000);


  let usdcToken = null;
  let redeemableToken = null;
  let usdcAccount = null;
  let redeemableAccount = null;

  
  it('Sets up the basic configuration', async () => {
    usdcToken = await createMint(provider);
    
    redeemableToken = await createMint(provider);
    
    usdcAccount = await createTokenAccount(
      provider,
      usdcToken.publicKey,
      provider.wallet.publicKey
    );
    
    redeemableAccount = await createTokenAccount(
      provider,
      redeemableToken.publicKey,
      provider.wallet.publicKey
    )

    await redeemableToken.mintTo(
      redeemableAccount,
      provider.wallet.publicKey,
      [],
      redeemableAmount.toString()
    );

    redeemableInfo = await getTokenAccount(
      provider,
      redeemableAccount
    );
    
    assert.ok(redeemableInfo.amount.eq(redeemableAmount));

    let bumps = new PoolBumps();

    const [idoAccount, idoAccountBump] = await anchor.web3.PublicKey.findProgramAddress(
      [Buffer.from(programName)],
      program.programId
    );
    bumps.idoAccount = idoAccountBump;

    const [poolUsdc, poolUsdcBump] = 
        await anchor.web3.PublicKey.findProgramAddress(
          [Buffer.from(programName), Buffer.from("pool_usdc")],
          program.programId
        );
    bumps.poolUsdc = poolUsdcBump;

    const [redeemableMint, redeemableMintBump] = 
          await anchor.web3.PublicKey.findProgramAddress(
            [Buffer.from(programName), Buffer.from("redeemable_mint")],
            program.programId
          );
    bumps.redeemableMint = redeemableMintBump;
    // Add your test here.
    const baseConfig = anchor.web3.Keypair.generate();
    await program.rpc.initialize(
      programName,
      bumps,
      {
      // programName,
      accounts: {
        idoAccount,
        redeemableMint, 
        poolUsdc: poolUsdc,
        usdcToken: usdcToken.publicKey,
        baseConfig: baseConfig.publicKey,
        tokenProgram: TOKEN_PROGRAM_ID,
        user: provider.wallet.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
        rent: anchor.web3.SYSVAR_RENT_PUBKEY
      },
      signers: [baseConfig]
    });
  });

  let userUsdc = null;
  // 10 usdc
  const firstDeposit = new anchor.BN(10_000_349);

  it('transfers usdc from user to program/pool', async () =>{
    const [redeemableMint] = await anchor.web3.PublicKey.findProgramAddress(
      [Buffer.from(programName), Buffer.from("redeemable_mint")],
      program.programId
    );

    const [poolUsdc] = await anchor.web3.PublicKey.findProgramAddress(
      [Buffer.from(programName), Buffer.from("pool_usdc")],
      program.programId
    )

    userUsdc = await Token.getAssociatedTokenAddress(
      ASSOCIATED_TOKEN_PROGRAM_ID,
      TOKEN_PROGRAM_ID,
      usdcToken.publicKey,
      program.provider.wallet.publicKey
    );
    let createUserUsdcInstr = Token.createAssociatedTokenAccountInstruction(
      ASSOCIATED_TOKEN_PROGRAM_ID,
      TOKEN_PROGRAM_ID,
      usdcToken.publicKey,
      userUsdc,
      program.provider.wallet.publicKey,
      program.provider.wallet.publicKey
    );
    let createUserUsdcTrans = new anchor.web3.Transaction().add(
      createUserUsdcInstr
    );
    await provider.send(createUserUsdcTrans);

    await usdcToken.mintTo(
      userUsdc,
      provider.wallet.publicKey,
      [],
      firstDeposit.toString()
    );

    userUsdcAccount = await getTokenAccount(provider, userUsdc);
    assert.ok(userUsdcAccount.amount.eq(firstDeposit));

    const [userRedeemable] = await anchor.web3.PublicKey.findProgramAddress(
      [
        provider.wallet.publicKey.toBuffer(),
        Buffer.from(programName),
        Buffer.from("user_redeemable")
      ],
      program.programId
    );

    const [idoAccount] = await anchor.web3.PublicKey.findProgramAddress(
      [Buffer.from(programName)],
      program.programId
    );

    try {
      const tx = await program.rpc.exchangeUsdcForRedeemable(firstDeposit, {
        accounts: {
          userAuthority: provider.wallet.publicKey,
          userUsdc,
          userRedeemable,
          idoAccount,
          usdcMint: usdcToken.publicKey,
          redeemableMint,
          // watermelonMint,
          poolUsdc,
          tokenProgram: TOKEN_PROGRAM_ID,
        },
        instructions: [
          program.instruction.initUserRedeemable({
            accounts: {
              userAuthority: provider.wallet.publicKey,
              userRedeemable,
              idoAccount,
              redeemableMint,
              systemProgram: anchor.web3.SystemProgram.programId,
              tokenProgram: TOKEN_PROGRAM_ID,
              rent: anchor.web3.SYSVAR_RENT_PUBKEY,
            },
          }),
        ],
        
      });
    } catch (err) {
      console.log("This is the error message: ", err.toString());
    }
    poolUsdcAccount = await getTokenAccount(provider, poolUsdc);
    assert.ok(poolUsdcAccount.amount.eq(firstDeposit));
  })

  function PoolBumps() {
    this.idoAccount;
    this.redeemableMint;
    this.poolUsdc;
  }


});
