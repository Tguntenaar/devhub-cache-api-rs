/**
 * Downloads all transactions for a specified NEAR account and saves them to individual JSON files.
 *
 * Each file is:
 * - Named after the block height used in the query parameter 'after_block'
 * - Stored in a folder named after the account (e.g., 'infrastructure-committee.near/')
 *
 * The download process:
 * 1. Starts from a specified block height
 * 2. Downloads transactions in batches
 * 3. Uses the last transaction's block height as the next starting point
 * 4. Continues until an empty response is received
 *
 * Example API endpoints:
 * - infrastructure-committee.near: .../txns?after_block=133636952
 * - devhub.near: .../txns?after_block=133636951
 * - events-committee.near: .../txns?after_block=133636951
 */

// the last devhub: https://api.nearblocks.io/v1/account/infrastructure-committee.near/txns?after_block=133636952&per_page=25&order=asc&page=1
// the last infra committee: https://api.nearblocks.io/v1/account/devhub.near/txns?after_block=133636951&per_page=25&order=asc&page=1
// the last events committee: https://api.nearblocks.io/v1/account/events-committee.near/txns?after_block=133636951&per_page=25&order=asc&page=1
const fs = require("fs");
const path = require("path");

const ACCOUNT = "devhub.near";
const BASE_URL = "https://api.nearblocks.io/v1/account";
const PER_PAGE = 25;
const API_KEY = "API_KEY";
const START_AFTER_BLOCK = 0;
const RECEIPT = false;

async function saveTransactions(blockHeight, transactions) {
  // Create a Blob containing the JSON data
  const outputDir = `./${ACCOUNT}${RECEIPT ? "-receipt" : ""}-${PER_PAGE}`;
  if (!fs.existsSync(outputDir)) {
    fs.mkdirSync(outputDir);
  }

  // Save file to disk
  const filePath = path.join(outputDir, `${blockHeight}.json`);
  await fs.promises.writeFile(filePath, JSON.stringify(transactions, null, 2));
  console.log(`Saved file: ${filePath}`);
}

async function fetchTransactions(afterBlock) {
  const url = `${BASE_URL}/${ACCOUNT}/txns?to=${ACCOUNT}&after_block=${afterBlock}&per_page=${PER_PAGE}&order=asc&page=1`;

  try {
    console.log(url);
    const response = await fetch(url, {
      headers: {
        Authorization: `Bearer ${API_KEY}`,
      },
    });
    const data = await response.json();
    return data.txns || [];
  } catch (error) {
    console.error(
      `Error fetching transactions after block ${afterBlock}:`,
      error.message
    );
    throw error;
  }
}

async function downloadAllTransactions() {
  let afterBlock = START_AFTER_BLOCK;
  let totalDownloaded = 0;

  while (true) {
    console.log(`Fetching transactions after block ${afterBlock}...`);

    const transactions = await fetchTransactions(afterBlock);

    if (transactions.length === 0) {
      console.log("No more transactions found. Download complete!");
      break;
    }

    await saveTransactions(afterBlock, transactions);
    totalDownloaded += transactions.length;

    // Update afterBlock to the block height of the last transaction
    const lastTx = transactions[transactions.length - 1];
    let index = RECEIPT ? "receipt_block" : "block";
    afterBlock = lastTx[index].block_height;
    console.log(`Next after_block: ${afterBlock}`);

    console.log(
      `Saved ${transactions.length} transactions. Total downloaded: ${totalDownloaded}`
    );

    // Add a small delay to avoid rate limiting
    await new Promise((resolve) => setTimeout(resolve, 1000));
  }
}

// Run the download
downloadAllTransactions()
  .then(() => console.log("Download completed successfully"))
  .catch((error) => console.error("Download failed:", error));
