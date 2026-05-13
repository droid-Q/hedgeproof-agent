require("@nomicfoundation/hardhat-ethers");

const ROBINHOOD_TESTNET_RPC_URL = process.env.ROBINHOOD_TESTNET_RPC_URL || "";
const ARBITRUM_SEPOLIA_RPC_URL = process.env.ARBITRUM_SEPOLIA_RPC_URL || "";
const DEPLOYER_PRIVATE_KEY = process.env.DEPLOYER_PRIVATE_KEY || "";

const accounts = DEPLOYER_PRIVATE_KEY ? [DEPLOYER_PRIVATE_KEY] : [];

module.exports = {
  solidity: {
    version: "0.8.24",
    settings: {
      optimizer: {
        enabled: true,
        runs: 200,
      },
    },
  },
  networks: {
    hardhat: {},
    robinhoodTestnet: {
      url: ROBINHOOD_TESTNET_RPC_URL || "http://127.0.0.1:8545",
      accounts,
    },
    arbitrumSepolia: {
      url: ARBITRUM_SEPOLIA_RPC_URL || "http://127.0.0.1:8545",
      accounts,
    },
  },
};
