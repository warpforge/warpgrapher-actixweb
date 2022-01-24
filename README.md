# warpgrapher-actixweb
Warpgrapher-actixweb integration example

Check out the [book](https://warpforge.github.io/warpgrapher/warpgrapher/quickstart.html) to learn more about Warpgrapher.

This example uses a NEO4J database, and expects the following environment variables to be set

```bash
export WG_NEO4J_HOST=127.0.0.1
export WG_NEO4J_READ_REPLICAS=127.0.0.1
export WG_NEO4J_PORT=7687
export WG_NEO4J_USER=neo4j
export WG_NEO4J_PASS=*MY-DB-PASS*
```

You can easily start a local NEO4J database for testing and development with docker

```
docker run --rm -p 7687:7687 -p 7474:7474 -e NEO4J_AUTH="${WG_NEO4J_USER}/${WG_NEO4J_PASS}" neo4j:4.1
```

Run the example
```
cargo run ./src/config.yaml
```
