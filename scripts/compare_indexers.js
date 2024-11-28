const fetch_from_pagoda = async () => {
  const body = {
    query:
      "query GetLatestSnapshot($offset: Int = 0, $limit: Int = 10, $where: polyprogrammist_near_devhub_prod_v1_proposals_with_latest_snapshot_bool_exp = {}) {\n    polyprogrammist_near_devhub_prod_v1_proposals_with_latest_snapshot(\n      offset: $offset\n      limit: $limit\n      order_by: {proposal_id: desc}\n      where: $where\n    ) {\n      author_id\n      block_height\n      name\n      category\n      summary\n      editor_id\n      proposal_id\n      ts\n      timeline\n      views\n      labels\n      linked_rfp\n    }\n    polyprogrammist_near_devhub_prod_v1_proposals_with_latest_snapshot_aggregate(\n      order_by: {proposal_id: desc}\n      where: $where\n    )  {\n      aggregate {\n        count\n      }\n    }\n  }",
    variables: {
      limit: 10,
      offset: 0,
      where: {},
    },
    operationName: "GetLatestSnapshot",
  };
  const response = await fetch(
    `https://near-queryapi.api.pagoda.co/v1/graphql`,
    {
      method: "POST",
      headers: { "x-hasura-role": "polyprogrammist_near" },
      body: JSON.stringify(body),
    }
  );
  let json = await response.json();
  // console.log(json);
  return json;
};

const fetch_from_cache = async () => {
  const response = await fetch(
    `https://devhub-cache-api-rs.fly.dev/proposals`,
    {
      method: "GET",
      headers: { "Content-Type": "application/json" },
    }
  );
  let json = await response.json();
  // console.log(json);
  return json;
};

const compare_results = async () => {
  const cache_result = await fetch_from_cache();
  const pagoda_result = await fetch_from_pagoda();
  let pagoda_records =
    pagoda_result.data
      .polyprogrammist_near_devhub_prod_v1_proposals_with_latest_snapshot;
  let cache_records = cache_result.records;
  let pagoda_total =
    pagoda_result.data
      .polyprogrammist_near_devhub_prod_v1_proposals_with_latest_snapshot_aggregate
      .aggregate;

  let cache_total = cache_result.total_records;

  // Check which proposals is not the same in both indexers
  console.log(
    "Proposals that are not the same in both indexers: ",
    pagoda_records
      .filter(
        (p) =>
          !cache_records.some(
            (c) =>
              JSON.parse(p.timeline).status === JSON.parse(c.timeline).status
          )
      )
      .map((p) => p.proposal_id)
  );

  console.log(
    "Pagoda ids: ",
    pagoda_records.map((p) => [p.proposal_id, JSON.parse(p.timeline).status])
  );
  console.log(
    "Cache ids: ",
    cache_records.map((c) => [c.proposal_id, JSON.parse(c.timeline).status])
  );

  console.log("pagoda_total, cache_total", pagoda_total, cache_total);
};

compare_results();
