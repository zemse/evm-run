#! /usr/bin/env node
const { BN } = require("ethereumjs-util");
const { parseCode } = require("./parseCode");
const { getVM } = require("./vm");
const { ethers } = require("ethers");
const { getOpcodeList } = require("./opcodes");

var argv = require("minimist")(process.argv.slice(2), {
  string: ["code"],
  boolean: ["activatePrecompiles"],
});

const code = argv.code ?? argv._[0];
// console.log(argv);
if (!code) throw new Error("No code provided");

const vm = getVM(
  argv.chain ?? "mainnet",
  argv.hardfork ?? "arrowGlacier",
  argv.activatePrecompiles ?? true,
  argv.rpc === true
    ? "https://eth-mainnet.alchemyapi.io/v2/BlFofLhaR2b18O08NFxUKPdBjHjRCj4P"
    : argv.rpc,
  argv.forkBlockNumber
);

const opcodeList = getOpcodeList(vm._common);

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
      display += `Stack: ${data.stack
        .map((val) => val.toString(16).toUpperCase())
        .slice()
        .reverse()}`;
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
      let returnHex = results.returnValue.toString("hex");
      let returnParsedStr;
      try {
        returnParsedStr = ethers.utils.toUtf8String("0x" + returnHex);
      } catch {}
      const printableLength = returnParsedStr
        .split("")
        .filter((val) =>
          val.match(/^[a-z0-9!"#$%&'()*+,.\/:;<=>?@\[\] ^_`{|}~-]*$/i)
        )
        .join("").length;
      console.log(
        `Returned: ${
          printableLength > 0 ? `"${returnParsedStr}" ` : ""
        }${returnHex}`
      );
      console.log(`gasUsed : ${results.gasUsed.toString()}`);
      if (results.exceptionError) {
        console.log(
          results.exceptionError.errorType,
          results.exceptionError.error
        );
      }
    })
    .catch(console.error);
}

main().catch(console.error);
