const { Common } = require("@ethereumjs/common");
const { VM } = require("@ethereumjs/vm");
const { BN } = require("ethereumjs-util");

const HttpProvider =
  require("hardhat/internal/core/providers/http").HttpProvider;
const JsonRpcClient =
  require("hardhat/internal/hardhat-network/jsonrpc/client").JsonRpcClient;
const ForkBlockchain =
  require("hardhat/internal/hardhat-network/provider/fork/ForkBlockchain").ForkBlockchain;
const ForkStateManager =
  require("hardhat/internal/hardhat-network/provider/fork/ForkStateManager").ForkStateManager;

function getVM(
  chain,
  hardfork,
  activatePrecompiles,
  mainnetForkRpc,
  forkBlockNumber = 14379250
) {
  const common = new Common({ chain, hardfork });

  const vmOptions = {
    common,
    activatePrecompiles,
  };

  if (mainnetForkRpc) {
    const httpProvider = new HttpProvider(mainnetForkRpc);
    // last arg is disk cache path
    const forkBlockNumberBN = new BN(forkBlockNumber);
    console.log(`forkBlockNumber: ${forkBlockNumberBN.toString(10)}`);
    const rpc = new JsonRpcClient(httpProvider, 1, forkBlockNumberBN, 2);
    vmOptions.blockchain = new ForkBlockchain(rpc, forkBlockNumberBN, common);
    vmOptions.stateManager = new ForkStateManager(rpc, forkBlockNumberBN);
  }

  return new VM(vmOptions);
}

module.exports = {
  getVM,
};
