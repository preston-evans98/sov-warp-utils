# Warp Utils

This package provides a simple utility to compute the warp route ID and token ID that the Sovereign SDK will generate for a given deployment. 

## Usage

```
Usage: sov-warp-utils --deployer <DEPLOYER> --token-address <TOKEN_ADDRESS>

Options:
  -d, --deployer <DEPLOYER>            The address that will be used to deploy the warp route on the Sovereign SDK chain
  -t, --token-address <TOKEN_ADDRESS>  The ethereum address of the wrapped token on the EVM chain
  -h, --help                           Print help
```

## Example
```
$ cargo run -- --deployer 0xD2C1bE33A0BcD2007136afD8Ed61CC7561aDa747 -token-address 0x4ed7c70F96B99c776995fB64377f0d4aB3B0e1C1
Warp Route ID: 0x9c081539d40ef7b02d359c5d694e006f0c1130097466cd22d062e07065c6987a
Token ID: token_195zght0wmhcx9j462jtj9lypdua4xw07r6jnjfjsddsmzeh2wswq8kfe5m
```
