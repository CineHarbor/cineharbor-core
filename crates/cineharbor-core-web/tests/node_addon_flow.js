// 端到端验证：web 薄客户端由 WASM core 拿到一条 catalog → meta → streams 数据。
//
// 前置：先构建 wasm 并生成 nodejs glue：
//   cd cineharbor-core
//   cargo build -p cineharbor-core-web --target wasm32-unknown-unknown
//   wasm-bindgen target/wasm32-unknown-unknown/debug/cineharbor_core_web.wasm \
//     --target nodejs --out-dir <GLUE_DIR>
// 运行：
//   CH_GLUE_DIR=<GLUE_DIR> node cineharbor-core/crates/cineharbor-core-web/tests/node_addon_flow.js
//
// 本脚本起一个 mock Stremio addon（node http server），再经 wasm 的
// addon_{manifest,catalog,meta,streams}_json 桥走 fetch 拉取，断言全链路。

const http = require("http");
const path = require("path");

const glueDir = process.env.CH_GLUE_DIR;
if (!glueDir) {
  console.error("缺少 CH_GLUE_DIR");
  process.exit(2);
}
const wasm = require(path.join(glueDir, "cineharbor_core_web.js"));

const manifest = {
  id: "mock.addon",
  version: "1.0.0",
  name: "Mock",
  resources: ["catalog", "meta", "stream"],
  types: ["movie"],
  catalogs: [{ type: "movie", id: "top", name: "Top" }],
};
const catalogBody = { metas: [{ id: "tt1", type: "movie", name: "Mock Movie" }] };
const metaBody = { meta: { id: "tt1", type: "movie", name: "Mock Movie" } };
const streamsBody = {
  streams: [{ title: "1080p", url: "https://cdn.test/a.m3u8" }],
};

const routes = {
  "/manifest.json": manifest,
  "/catalog/movie/top.json": catalogBody,
  "/meta/movie/tt1.json": metaBody,
  "/stream/movie/tt1.json": streamsBody,
};

const server = http.createServer((req, res) => {
  const url = req.url.split("?")[0];
  const body = routes[url];
  if (!body) {
    res.statusCode = 404;
    res.end("{}");
    return;
  }
  res.setHeader("content-type", "application/json");
  res.end(JSON.stringify(body));
});

server.listen(0, "127.0.0.1", async () => {
  const base = `http://127.0.0.1:${server.address().port}`;
  try {
    const got = JSON.parse(await wasm.addon_manifest_json(base));
    if (got.id !== "mock.addon") throw new Error(`manifest 不符: ${JSON.stringify(got)}`);

    const catalog = JSON.parse(
      await wasm.addon_catalog_json(base, "movie", "top", undefined, undefined, undefined),
    );
    if (catalog.metas?.length !== 1 || catalog.metas[0].id !== "tt1") {
      throw new Error(`catalog 不符: ${JSON.stringify(catalog)}`);
    }

    const meta = JSON.parse(await wasm.addon_meta_json(base, "movie", "tt1"));
    if (meta.meta?.id !== "tt1") throw new Error(`meta 不符: ${JSON.stringify(meta)}`);

    const streams = JSON.parse(await wasm.addon_streams_json(base, "movie", "tt1"));
    if (streams.streams?.length !== 1 || streams.streams[0].url !== "https://cdn.test/a.m3u8") {
      throw new Error(`streams 不符: ${JSON.stringify(streams)}`);
    }

    console.log("INTEGRATION OK: catalog -> meta -> streams 经 WASM core + fetch 全通");
  } catch (error) {
    console.error("INTEGRATION FAIL:", error);
    process.exitCode = 1;
  } finally {
    server.close();
  }
});