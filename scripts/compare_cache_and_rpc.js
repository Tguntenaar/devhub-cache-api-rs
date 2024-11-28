const fetch_from_rpc = async (proposal_id) => {
  let args_base64 = Buffer.from(JSON.stringify({ proposal_id })).toString(
    "base64"
  );
  // console.log(args_base64);

  const body = {
    jsonrpc: "2.0",
    id: "dontcare",
    method: "query",
    params: {
      request_type: "call_function",
      finality: "final",
      account_id: "devhub.near",
      method_name: "get_proposal",
      args_base64: args_base64,
    },
  };

  const response = await fetch(ARCHIVAL_RPC_URL, {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
    },
    body: JSON.stringify(body),
  });
  let json = await response.json();
  let result = JSON.parse(Buffer.from(json.result.result).toString("utf-8"));
  // console.log(result.snapshot.timeline.status);
  return result;
};

const fetch_from_cache = async (proposal_id) => {
  const response = await fetch(
    `${CACHE_API_URL}proposal/${proposal_id}/snapshots`,
    {
      method: "GET",
      headers: { "Content-Type": "application/json" },
    }
  );
  let json = await response.json();
  return json;
};

const compare_results = async (id) => {
  const rpc_result = await fetch_from_rpc(id);
  const cache_results = await fetch_from_cache(id);

  let cache_snapshots = cache_results.map(parse_timeline);
  let rpc_snapshots = [rpc_result.snapshot, ...rpc_result.snapshot_history].map(
    (item) => ({ ...item, proposal_id: rpc_result.id })
  );

  // get_only_useful_fields for the snapshots and compare them
  let cache_only_useful_fields = cache_snapshots.map(get_only_useful_fields);
  let rpc_only_useful_fields = rpc_snapshots.map(get_only_useful_fields);

  // console.log(
  //   "cache_only_useful_fields",
  //   cache_only_useful_fields.length,
  //   "rpc_only_useful_fields",
  //   rpc_only_useful_fields.length
  // );

  if (cache_only_useful_fields.length !== rpc_only_useful_fields.length) {
    console.log(
      "Lengths are not the same at proposal ",
      rpc_result.id,
      "cache",
      cache_only_useful_fields.length,
      "!= rpc ",
      rpc_only_useful_fields.length
    );
    return;
  }

  for (let i = 0; i < cache_only_useful_fields.length; i++) {
    console.log(
      `proposal_id: ${cache_only_useful_fields[i].proposal_id} snapshot ${i}`
    );
    print_object_diff(cache_only_useful_fields[i], rpc_only_useful_fields[i]);
  }
};

const parse_timeline = (snapshot) => {
  return {
    ...snapshot,
    timeline: JSON.parse(snapshot.timeline),
  };
};

const get_only_useful_fields = (snapshot) => {
  return {
    proposal_id: snapshot.proposal_id,
    status: snapshot.timeline.status,
    timeline: snapshot.timeline,
    // timestamp and block height are not in the rpc response
    // ts: snapshot.ts,
    // block_height: snapshot.block_height,
  };
};

const print_object_diff = (obj1, obj2) => {
  // Find the first key that is different
  // console.log(Object.keys(obj1));
  let key = Object.keys(obj1).find((key) => {
    if (key === "timeline") {
      return obj1[key].status !== obj2[key].status;
    }
    return obj1[key] !== obj2[key];
  });
  if (key) {
    console.log(" Diff: ", key, obj1[key], obj2[key]);
  } else {
    console.log(" No diff found");
  }
};

const START_ID = 250;
const END_ID = 260;
const ARCHIVAL_RPC_URL = "https://archival-rpc.mainnet.near.org/";
const CACHE_API_URL = "https://devhub-cache-api-rs.fly.dev/";

const runComparisons = async () => {
  for (let i = START_ID; i < END_ID; i++) {
    await compare_results(i);
    // Optional: Add a small delay between requests to avoid rate limiting
    await new Promise((resolve) => setTimeout(resolve, 1000)); // 1 second delay
  }
};

// Call the async function
runComparisons().catch(console.error);
