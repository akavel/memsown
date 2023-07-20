In the context of <!-- use case/story uc or component co -->
 the need to cache miniatures of photos and metadata about backup status
 (to allow quick display/browsing esp. when not connected to backup backends),
facing <!-- concern c -->
 a dilemma what storage to use,

we decided for **<!-- option o -->
 SQLite**,
and resigned from <!-- options o2 to oN -->
 flat files storage,

to achieve <!-- quality q -->
* ease of querying (notably SQL),
* ease of prototyping (notably SQL, including inherent possibilities of batch-processing of records),
* conciseness of storage (one file instead of many files),
* easy decent robustness and fail-proofness (based on reputation),

accepting <!-- downside d -->
* storage space overhead/bloat,
* possible need to find a better optimized format at some time in the future (but: "premature optimization... etc.").

## Extra notes
- also as a bonus got availability of third-party tools for browsing SQLite DBs.
