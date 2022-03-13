const Common = require("@ethereumjs/common").default;
const VM = require("@ethereumjs/vm").default;
const { BN } = require("ethereumjs-util");

const HttpProvider =
  require("hardhat/internal/core/providers/http").HttpProvider;
const JsonRpcClient =
  require("hardhat/internal/hardhat-network/jsonrpc/client").JsonRpcClient;
const ForkBlockchain =
  require("hardhat/internal/hardhat-network/provider/fork/ForkBlockchain").ForkBlockchain;
const ForkStateManager =
  require("hardhat/internal/hardhat-network/provider/fork/ForkStateManager").ForkStateManager;

function getVM(chain, hardfork, activatePrecompiles, mainnetForkRpc) {
  const common = new Common({ chain, hardfork });

  const vmOptions = {
    common,
    activatePrecompiles,
  };

  if (mainnetForkRpc) {
    const httpProvider = new HttpProvider(
      mainnetForkRpc // "https://eth-mainnet.alchemyapi.io/v2/BlFofLhaR2b18O08NFxUKPdBjHjRCj4P"
    );
    // last arg is disk cache path
    const forkBlockNumber = new BN(14379250);
    const rpc = new JsonRpcClient(httpProvider, 1, forkBlockNumber, 2);
    vmOptions.blockchain = new ForkBlockchain(rpc, forkBlockNumber, common);
    vmOptions.stateManager = new ForkStateManager(rpc, forkBlockNumber);
  }

  return new VM(vmOptions);
}

module.exports = {
  getVM,
};
