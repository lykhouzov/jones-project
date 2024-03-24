# Jones' Project
Toy transaction engine

## Description and thoughts
Current implementation is single-threaded.
Here are my assumption that I have
- input csv file has strict format, that means if there extra space, th line will have an error, but the rest of file should be processed
- only `deposit` and `withdrawal` transactions can have amount. That means if a `dispute` transaction has an `amount` populated that row will be skiped.
- a `dispute` can be open for `deposit` and `withdrawal` transactions _only_. 
- If a `dispute` transaction refers to other then `deposit` or `withdrawal` it will be ignored.Current implementation does not save it.
- `dispute` can be opened either for `deposit` or `withddrawal` transaction.
-- if a `dispute` for `withdrawal` transaction it will add an `amount` to the held withiut reducing `available`
- Current implementation does not save other than `deposit` and `withdrawal` transactions but tracks `dispute`/`resolv`e/`chargeback` as a state of a transaction
- There is a logging implemented. It is disabled by default, but for testing purposes could be enabled with `--features logging` flag. Bear in mind, that in this case logs will be output to stdout, meaning it is not what is required by the task but good for testing. On production logs should be transfered to a log collector like grafana or ELK stack or equivalent

## What can be additionally implemented
- Separate  reading if a file and processing transactions by sending parsed row(s) to a queue, and then have separate process(es) that will consume the queue.
- Currently when a transaction in process it blocks reading of the file. To unblock that it could be done like: 
-- read file by chunks
-- send a chunk to separate process to parse and send to queue
-- process a queue in separate process

- The engine could be done as a simple microservice. How I see it:
-- single transaction enpoint `async process(tx: Transaction) -> impl Response` handled by Axum/Actix framework. This enpoint receives single transaction data and sends it to a queue
-- bulk transaction enpoint `async process_bluk(txs: Vec<Transaction>)-> impl Response`. This endpoint receives a list of transactions, like in the task. Send them to queue
-- the rest of implementation looks the same.

## The prossible issues
When a parallel/async process introduced there might be an issue of handling transactions chonologically, meaning that transactions can appear in a queue in different order as input to the service. That will lead to an issues like a `dispute` will be processed before `withdrawal` or `deposit`, although last ones chonologically appears ealier.


## How to run
```fish
cargo run -- transactions.csv > accounts.csv
```
for debug purpose only to see logs run
```fish
export RUST_LOG=debug
cargo run -- --logger transactions.csv
```

### Input
CSV file
```csv
type, client, tx, amount
deposit,1,1,1.0
deposit,2,2,2.0
deposit,1,3,2.0
withdrawal,1,4,1.5
withdrawal,2,5,3.0
```

### Output
```csv
client, available, held, total, locked
1,1.5,0,1.5,false
2,2,0,2,false
```

## Generate an example file
```fish
cargo run --bin generator 10 100
```
```fish
Usage: generator <NUM_CLIENTS> <NUM_LINES>
```

