const { assert } = require("chai");
const { account, describeWithAnon } = require('../helpers/utils');
// const UniswapV2ERC20 = require('../node_modules/@uniswap/v2-core/build/UniswapV2ERC20.json');
const ERC20 = require('../node_modules/@openzeppelin/contracts/build/contracts/ERC20.json');
const contract = require("@truffle/contract");

describeWithAnon("Allowance test", async (context) => {
  it("should compute allowance", async () => {
    const web3 = context.web3;

    let erc = contract({
      abi: ERC20.abi,
      unlinked_binary: ERC20.bytecode,
    });
    erc.setProvider(web3.currentProvider);

    const v = web3.utils.toWei('10', 'ether');
    let c = await erc.new({ from: account });

    // create with value
    const approvalAccount = '0xc0ffee254729296a45a3885639AC7E10F9d54979';
    await c.approve(approvalAccount, v, { from: account });

    const allowance = await c.allowance.call(account, approvalAccount, { from: account });
    assert.equal(allowance, v);
  });
});
