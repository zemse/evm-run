#! /usr/bin/env node
const { BN } = require("ethereumjs-util");
const { getOpcodesForHF } = require("@ethereumjs/vm/dist/evm/opcodes");
const { parseCode } = require("./parseCode");
const { getVM } = require("./vm");

var argv = require("minimist")(process.argv.slice(2), {
  string: ["code"],
  boolean: ["activatePrecompiles"],
});

const code = argv.code ?? argv._[0];
// console.log(argv);
if (!code) throw new Error("No code provided");

const vm = getVM(
  argv.chain ?? "mainnet",
  argv.hardfork ?? "berlin",
  argv.activatePrecompiles ?? true,
  argv.rpc
);

const opcodeList = Array.from(getOpcodesForHF(vm._common).values())
  .map((entry) => [entry.fullName, entry.code.toString(16)])
  .sort((entry1, entry2) => (entry1[0].length > entry2[0].length ? -1 : 1));

async function main() {
  const codeBuff = Buffer.from(await parseCode(code, opcodeList), "hex");

  if (argv.dest) {
    codeUint8Array = new Uint8Array(codeBuff);
    console.log(codeUint8Array);
  }

  let displayOpcodeMaxLength = 0;
  let displayStackMaxLength = 0;
  vm.on("step", function (data) {
    // console.log(data)
    let opcode = opcodeList.find((entry) => entry[0] === data.opcode.name)[1];
    let display = `${opcode} ${data.opcode.name}`;
    displayOpcodeMaxLength = Math.max(displayOpcodeMaxLength, display.length);
    if (data.stack.length) {
      display += " ".repeat(displayOpcodeMaxLength - display.length + 1);
      display += `Stack: ${data.stack.map((val) =>
        val.toString(16).toUpperCase()
      )}`;
    }
    displayStackMaxLength = Math.max(displayStackMaxLength, display.length);
    if (data.memory.length) {
      display += " ".repeat(displayStackMaxLength - display.length + 1);
      display += `Memory: ${data.memory.toString("hex")}`;
    }

    console.log(display);
  });

  if (argv.data === true) argv.data = "";
  if (typeof argv.data === "string" && argv.data.slice(0, 2) === "0x") {
    argv.data = argv.data.slice(2);
  }

  vm.runCode({
    code: Buffer.from(codeBuff, "hex"),
    gasLimit: new BN(argv.gasLimit ?? 0xffff),
    value: new BN(argv.value ?? 0x0),
    data: Buffer.from(
      (argv.data === true ? "" : String(argv.data)) ?? "0x",
      "hex"
    ),
  })
    .then((results) => {
      console.log(`Returned: ${results.returnValue.toString("hex")}`);
      console.log(`gasUsed : ${results.gasUsed.toString()}`);
    })
    .catch(console.error);
}

main().catch(console.error);
