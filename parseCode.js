const fs = require("fs-extra");
const { BN } = require("ethereumjs-util");
const axios = require("axios");

// returns evm code in hex
async function parseCode(code, opcodeList) {
  if (typeof code === "number") {
    if (code + 1 === code) throw new Error("Please use --code to pass code");
    code = String(code);
    if (code.length % 2 !== 0) code = "0" + code;
  }

  if (typeof code === "string" && code.slice(0, 4) === "http") {
    // reading code if url is provided
    code = await axios.get(code).then((res) => res.data);
  } else if (code.length < 100 && fs.existsSync(code)) {
    // reading file if a path is provided
    code = fs.readFileSync(code, "utf8");
  }

  // .map((entry) => [entry[1].fullName, entry[0]])
  if (code.slice(0, 2) === "0x") code = code.slice(2);

  // removing comments
  code = code
    // remove all occurences of 0x
    .split("0x")
    .join("")
    .split("0X")
    .join("")
    .split("\n")
    .map((line) => line.split("//")[0])
    .map((line) => line.split("#")[0])
    .filter((line) => !!line)
    .map((line) =>
      line
        .split(" ")
        .filter((word) => !!word)
        .map((word) =>
          // parse base ten numbers
          word.slice(0, 2) == "0t" ? new BN(word.slice(2)).toString(16) : word
        )
        .map(replaceAliasWithOpcode.bind(null, opcodeList))
        .map((word) => (word.length === 1 ? "0" + word : word))

        .join("")
    )
    .join("");

  console.log("code", code);
  return code;
}

function replaceAliasWithOpcode(opcodeList, code) {
  code = String(code).toUpperCase();

  for (opcode of opcodeList) {
    code = code
      .split(opcode[0])
      .join(opcode[1].length === 1 ? "0" + opcode[1] : opcode[1]);
  }

  if (!code.match(/^[a-fA-F0-9]+$/)) {
    const similarCodes = opcodeList
      .filter((entry) => entry[0].includes(code))
      .map((entry) => entry[0]);
    throw new Error(
      "Invalid code: " +
        code +
        ".\nSimilar codes: " +
        similarCodes.join(", ") +
        ". Please use something from these instead."
    );
  }

  return code;
}

module.exports = { parseCode };
