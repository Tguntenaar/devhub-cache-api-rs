// Store account_id and block_height of post
// Example:
// discussions.webassembly.community.devhub.near 1123124
// announcements.webassembly.community.devhub.near 1123125

// Each account announcements... or discussions.. need to keep track of the last
// block height they posted the indexer is up to date with. in proposals and rfps
// this is shared between the two since it only track the devhub.near contract account.
// This is basically done by the post table.

// Challenge:
// 1. get them up to date with history
// 2. keeping them up to date
