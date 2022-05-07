const Common = require("@ethereumjs/common").default;
const { getOpcodesForHF } = require("@ethereumjs/vm/dist/evm/opcodes");

function getOpcodeList(common) {
  return Array.from(getOpcodesForHF(common).opcodes.values())
    .map((entry) => [entry.fullName, entry.code.toString(16)])
    .sort((entry1, entry2) => (entry1[0].length > entry2[0].length ? -1 : 1));
}

const defaultOpcodeList = getOpcodeList(
  new Common({ chain: "mainnet", hardfork: "arrowGlacier" })
);

module.exports = {
  getOpcodeList,
  defaultOpcodeList,
};
