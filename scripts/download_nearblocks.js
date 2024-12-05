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
const PER_PAGE = 50;
const API_KEY = process.env.NEARBLOCKS_API_KEY;
if (!API_KEY) {
  throw new Error("NEARBLOCKS_API_KEY environment variable is required");
}

const CURSOR = "";
const RECEIPT = false; // Can't use receipt because it's not supported by the API after_block only checks after the block

async function saveTransactions(cursor, transactions) {
  // Create a Blob containing the JSON data
  const outputDir = `./${ACCOUNT}${
    RECEIPT ? "-receipt" : ""
  }-${PER_PAGE}-cursor`;
  if (!fs.existsSync(outputDir)) {
    fs.mkdirSync(outputDir);
  }

  // Save file to disk
  const filePath = path.join(outputDir, `${cursor ? cursor : "0"}.json`);
  await fs.promises.writeFile(filePath, JSON.stringify(transactions, null, 2));
  console.log(`Saved file: ${filePath}`);
}

async function fetchTransactions(cursor = "") {
  // Initial start at block 0 and use cursor afterwards
  const url = cursor
    ? `${BASE_URL}/${ACCOUNT}/txns?per_page=${PER_PAGE}&order=asc&cursor=${cursor}`
    : `${BASE_URL}/${ACCOUNT}/txns?after_block=0&per_page=${PER_PAGE}&order=asc&page=1`;

  try {
    console.log(url);
    const response = await fetch(url, {
      headers: {
        Authorization: `Bearer ${API_KEY}`,
      },
    });
    const data = await response.json();
    return data || [];
  } catch (error) {
    console.error(
      `Error fetching transactions after block ${afterBlock}:`,
      error.message
    );
    throw error;
  }
}

async function downloadAllTransactions() {
  let cursor = CURSOR;
  let totalDownloaded = 0;

  while (true) {
    console.log(`Fetching transactions with cursor "${cursor}"...`);

    const data = await fetchTransactions(cursor);
    let transactions = data.txns;

    if (transactions.length === 0) {
      console.log("No more transactions found. Download complete!");
      break;
    }

    await saveTransactions(cursor, transactions);
    totalDownloaded += transactions.length;

    cursor = data.cursor;
    console.log(`Next cursor: ${cursor}`);

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
