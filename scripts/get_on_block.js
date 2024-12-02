const fetch_from_rpc = async (proposal_id, block_height) => {
  let args_base64 = Buffer.from(JSON.stringify({ proposal_id })).toString(
    "base64"
  );
  console.log(args_base64);

  const body = {
    jsonrpc: "2.0",
    id: "dontcare",
    method: "query",
    params: {
      request_type: "call_function",
      block_id: block_height,
      account_id: "devhub.near",
      method_name: "get_proposal",
      args_base64: args_base64,
    },
  };
  const response = await fetch(`https://archival-rpc.mainnet.near.org/`, {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
    },
    body: JSON.stringify(body),
  });
  let json = await response.json();
  // console.log(json);
  // create string from buffer json.result.result
  let result = JSON.parse(Buffer.from(json.result.result).toString("utf-8"));
  // console.log(result);
  console.log(result.snapshot.timeline.status);
  return result;
};

fetch_from_rpc(250, 132540186);
