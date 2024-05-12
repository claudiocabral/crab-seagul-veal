# Crab Squid Veal

This is a toy project aimed at exercising some Rust muscle.
It takes a CSV file as an input containing data similar to what one could find
in a ledger and outputs the resulting client accounts in stdout.

The following operations are supported in he input file:
* Deposits: Increase the client's available funds by the amount specified in the
  transaction. The operation fails and is ignored in case of overflow, though
  that's unlikely since our numerical type represents very large numbers.
* Withdrawals: Decrease the client's available funds by the amount specified in
  the transaction. The operation fails if the client's available funds are
  smaller then the transaction amount or if the client account is frozen.
* Disputes: The client's available funds decrease by the amount specified in the
  transaction whilst the client's held funds increase by that same amount.
  Mismatched client ids, underflows of available funds or overflows of held
  funds all cause the operation to fail without modifying the client account in
  any way. Only deposits in an Ok state (in other words, not Disputed or
  Chargedback) can be disputed. Attempts to do otherwise will fail without
  modifying the client account. 
* Resolves: The client's held funds decrease by the amount specified in the
  transaction whilst the client's available funds increase by that same amount.
  Mismatched client ids, overflows of available funds or underflows of held
  funds all cause the operation to fail without modifying the client account in
  any way. Only deposits in a Disputed state (in other words, not Ok or
  Chargedback) can be resolved. Attempts to do otherwise will fail without
  modifying the client account. 
* Chargebacks: The client's held funds decrease by the amount specified in the
  transaction and the client account is marked as frozen.
  Mismatched client ids will cause the operation to fail without modifying the
  client account in any way. Only deposits in a Disputed state (in other
  words, not Ok or Chargedback) can be chargedback. Attempts to do otherwise will
  fail without modifying the client account. 

### Correctness 

* All withdrawals and deposits have a unique transaction ID. Repeated
  transaction IDs are ignored.
* Input files with a bad header will generate no transactions. Records that
  can't be properly parsed are ignored.
* Transaction errors are verified with unittests.
* CSV errors are verified with integration tests.
