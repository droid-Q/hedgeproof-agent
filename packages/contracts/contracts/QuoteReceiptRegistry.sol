// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

contract QuoteReceiptRegistry {
    struct QuoteReceipt {
        address requester;
        bytes32 quoteHash;
        bytes32 summaryHash;
        uint64 expiresAt;
        uint128 budgetCeilingUsd;
        string riskTag;
        string chainHint;
        uint64 createdAt;
    }

    mapping(bytes32 => QuoteReceipt) private receipts;

    event QuoteReceiptCreated(
        bytes32 indexed quoteId,
        address indexed requester,
        bytes32 quoteHash,
        bytes32 summaryHash,
        uint64 expiresAt,
        uint128 budgetCeilingUsd,
        string riskTag,
        string chainHint
    );

    error EmptyQuoteId();
    error EmptyQuoteHash();
    error EmptySummaryHash();
    error InvalidExpiry();
    error InvalidBudget();
    error ReceiptAlreadyExists();
    error ReceiptNotFound();

    function createReceipt(
        bytes32 quoteId,
        bytes32 quoteHash,
        bytes32 summaryHash,
        uint64 expiresAt,
        uint128 budgetCeilingUsd,
        string calldata riskTag,
        string calldata chainHint
    ) external {
        if (quoteId == bytes32(0)) revert EmptyQuoteId();
        if (quoteHash == bytes32(0)) revert EmptyQuoteHash();
        if (summaryHash == bytes32(0)) revert EmptySummaryHash();
        if (expiresAt <= block.timestamp) revert InvalidExpiry();
        if (budgetCeilingUsd == 0) revert InvalidBudget();
        if (receipts[quoteId].requester != address(0)) revert ReceiptAlreadyExists();

        receipts[quoteId] = QuoteReceipt({
            requester: msg.sender,
            quoteHash: quoteHash,
            summaryHash: summaryHash,
            expiresAt: expiresAt,
            budgetCeilingUsd: budgetCeilingUsd,
            riskTag: riskTag,
            chainHint: chainHint,
            createdAt: uint64(block.timestamp)
        });

        emit QuoteReceiptCreated(
            quoteId,
            msg.sender,
            quoteHash,
            summaryHash,
            expiresAt,
            budgetCeilingUsd,
            riskTag,
            chainHint
        );
    }

    function getReceipt(bytes32 quoteId) external view returns (QuoteReceipt memory receipt) {
        receipt = receipts[quoteId];
        if (receipt.requester == address(0)) revert ReceiptNotFound();
    }

    function isValid(bytes32 quoteId, bytes32 quoteHash) external view returns (bool) {
        QuoteReceipt memory receipt = receipts[quoteId];
        return receipt.requester != address(0)
            && receipt.quoteHash == quoteHash
            && receipt.expiresAt >= block.timestamp;
    }
}
