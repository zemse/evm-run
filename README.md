# evm-run

Run evm code from console or from file, on local env or mainnet fork.

## Setup

Clone this repo and run the following command:

```
cargo install --path .
```

## Usage

### Run code directly

Calculates square

```
$ evm-run --code 348002600052596000f3 --value 3
34 CALLVALUE
80 DUP1      Stack: 3
2 MUL        Stack: 3,3
60 PUSH1     Stack: 9
52 MSTORE    Stack: 0,9
59 MSIZE                Memory: 0000000000000000000000000000000000000000000000000000000000000009
60 PUSH1     Stack: 20  Memory: 0000000000000000000000000000000000000000000000000000000000000009
f3 RETURN    Stack: 0,20 Memory: 0000000000000000000000000000000000000000000000000000000000000009
Returned: 0000000000000000000000000000000000000000000000000000000000000009
gasUsed : 24
```

### Run pseudo code

```
$ evm-run --code 'push1 2 push1 3 add stop'
code 600260030100
60 PUSH1
60 PUSH1 Stack: 2
1 ADD    Stack: 3,2
0 STOP   Stack: 5
Returned:
gasUsed : 9
```

### Mainnet fork

Run evm code against mainnet fork, to get balance of vitalik.

```
% evm-run 363d600C373d51313d52593df3 --calldata d8dA6BF26964aF9D7eEd9e03E53415D37aA96045 --rpc https://eth-mainnet.alchemyapi.io/v2/<API-KEY> --block 14379250
36 CALLDATASIZE
3d RETURNDATASIZE Stack: 14
60 PUSH1          Stack: 0,14
37 CALLDATACOPY   Stack: C,0,14
3d RETURNDATASIZE               Memory: 000000000000000000000000d8da6bf26964af9d7eed9e03e53415d37aa96045
51 MLOAD          Stack: 0      Memory: 000000000000000000000000d8da6bf26964af9d7eed9e03e53415d37aa96045
31 BALANCE        Stack: D8DA6BF26964AF9D7EED9E03E53415D37AA96045 Memory: 000000000000000000000000d8da6bf26964af9d7eed9e03e53415d37aa96045
3d RETURNDATASIZE Stack: E034B8C125409C0113                       Memory: 000000000000000000000000d8da6bf26964af9d7eed9e03e53415d37aa96045
52 MSTORE         Stack: 0,E034B8C125409C0113                     Memory: 000000000000000000000000d8da6bf26964af9d7eed9e03e53415d37aa96045
59 MSIZE                                                          Memory: 0000000000000000000000000000000000000000000000e034b8c125409c0113
3d RETURNDATASIZE Stack: 20                                       Memory: 0000000000000000000000000000000000000000000000e034b8c125409c0113
f3 RETURN         Stack: 0,20                                     Memory: 0000000000000000000000000000000000000000000000e034b8c125409c0113
Returned: 0000000000000000000000000000000000000000000000e034b8c125409c0113
gasUsed : 2630
```

### Run block

```
evm-run --block 20132136 --rpc https://eth-mainnet.g.alchemy.com/v2/<API-KEY>
running block
████████████░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░ 1589329/12195104
```