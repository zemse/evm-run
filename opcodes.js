const { Common } = require("@ethereumjs/common");
const { getOpcodesForHF } = require("@ethereumjs/evm/dist/opcodes");

function getOpcodeList(common) {
  return Array.from(getOpcodesForHF(common).opcodes.values())
    .map((entry) => [entry.fullName, entry.code.toString(16)])
    .sort((entry1, entry2) => (entry1[0].length > entry2[0].length ? -1 : 1));
}

const defaultChain = "mainnet";
const defaultHardfork = "shanghai";

const defaultOpcodeList = getOpcodeList(
  new Common({ chain: defaultChain, hardfork: defaultHardfork })
);

module.exports = {
  getOpcodeList,
  defaultOpcodeList,
  defaultChain,
  defaultHardfork,
};
