const { expect } = require("chai");
const { ethers } = require("hardhat");

describe("QuoteReceiptRegistry", function () {
  async function deployRegistry() {
    const Registry = await ethers.getContractFactory("QuoteReceiptRegistry");
    const registry = await Registry.deploy();
    await registry.waitForDeployment();
    return registry;
  }

  const quoteId = ethers.id("quote-1");
  const quoteHash = ethers.id("quote-hash");
  const summaryHash = ethers.id("summary-hash");

  it("creates and validates a quote receipt", async function () {
    const registry = await deployRegistry();
    const now = (await ethers.provider.getBlock("latest")).timestamp;
    const expiresAt = now + 3600;

    await registry.createReceipt(
      quoteId,
      quoteHash,
      summaryHash,
      expiresAt,
      950,
      "RWA_EARNINGS_GAP",
      "Robinhood Chain Testnet",
    );

    expect(await registry.isValid(quoteId, quoteHash)).to.equal(true);
    expect(await registry.isValid(quoteId, ethers.id("tampered"))).to.equal(false);

    const receipt = await registry.getReceipt(quoteId);
    expect(receipt.quoteHash).to.equal(quoteHash);
    expect(receipt.summaryHash).to.equal(summaryHash);
    expect(receipt.budgetCeilingUsd).to.equal(950n);
    expect(receipt.riskTag).to.equal("RWA_EARNINGS_GAP");
  });

  it("rejects duplicate quote ids", async function () {
    const registry = await deployRegistry();
    const now = (await ethers.provider.getBlock("latest")).timestamp;

    await registry.createReceipt(
      quoteId,
      quoteHash,
      summaryHash,
      now + 3600,
      950,
      "RWA_EARNINGS_GAP",
      "Robinhood Chain Testnet",
    );

    await expectCustomError(
      registry.createReceipt(
        quoteId,
        ethers.id("other"),
        summaryHash,
        now + 7200,
        1000,
        "RWA_EARNINGS_GAP",
        "Robinhood Chain Testnet",
      ),
      "ReceiptAlreadyExists",
    );
  });

  it("rejects expired receipts", async function () {
    const registry = await deployRegistry();
    const now = (await ethers.provider.getBlock("latest")).timestamp;

    await expectCustomError(
      registry.createReceipt(
        quoteId,
        quoteHash,
        summaryHash,
        now,
        950,
        "RWA_EARNINGS_GAP",
        "Robinhood Chain Testnet",
      ),
      "InvalidExpiry",
    );
  });
});

async function expectCustomError(promise, errorName) {
  try {
    await promise;
    expect.fail(`Expected ${errorName}`);
  } catch (error) {
    expect(error.message).to.include(errorName);
  }
}
