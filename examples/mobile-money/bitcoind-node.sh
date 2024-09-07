#!/bin/bash

#
# Helper functions
#

address=bcrt1qlhwg8036lga3c2t4pmmc6wf49f8t0m5gshjzpj
p2wkh=0014fddc83be3afa3b1c29750ef78d39352a4eb7ee88
rpcuser=regtest
rpcpassword=regtest
rpcport=18443

# pull and run bitcoind
function run-bitcoind () {
    docker compose -f "./docker-compose.yml" up -d 
}


# run-in-node: Run a command inside a docker container, using the bash shell
function run-in-node () {
	docker exec "$1" /bin/bash -c "${@:2}"
}

# wait-for-cmd: Run a command repeatedly until it completes/exits successfuly
function wait-for-cmd () {
		until "${@}" 2>&1
		do
			echo -n "."
			sleep 1
		done
		echo
}

# wait-for-node: Run a command repeatedly until it completes successfully, inside a container
# Combining wait-for-cmd and run-in-node
function wait-for-node () {
	wait-for-cmd run-in-node $1 "${@:2}"
}

function mine-block () {
	newaddress=$(wait-for-node bitcoindnode "bitcoin-cli -regtest -rpcwait -rpcport=$rpcport  -rpcuser=$rpcuser -rpcpassword=$rpcpassword  getnewaddress")
	echo $newaddress
	wait-for-node bitcoindnode "bitcoin-cli -regtest -rpcwait -rpcport=$rpcport  -rpcuser=$rpcuser -rpcpassword=$rpcpassword generatetoaddress 6 $newaddress"
	
}
function mine-first-block () {
	newaddress=$(wait-for-node bitcoindnode "bitcoin-cli -regtest -rpcwait -rpcport=$rpcport  -rpcuser=$rpcuser -rpcpassword=$rpcpassword  getnewaddress")
	echo $newaddress
	wait-for-node bitcoindnode "bitcoin-cli -regtest -rpcwait -rpcport=$rpcport  -rpcuser=$rpcuser -rpcpassword=$rpcpassword generatetoaddress 101 $newaddress"
	
}


# Start the demo
echo "Starting End To End Test"

echo "======================================================"
docker compose -f "./docker-compose.yml" up -d  2>&1 &
echo -n "- Waiting for bitcoind startup..."
wait-for-node bitcoindnode "bitcoin-cli -regtest -rpcwait -rpcport=$rpcport  -rpcuser=$rpcuser -rpcpassword=$rpcpassword getblockchaininfo"
wait-for-node bitcoindnode "bitcoin-cli -regtest -rpcwait -rpcport=$rpcport  -rpcuser=$rpcuser -rpcpassword=$rpcpassword createwallet regtest > /dev/null"

echo -n "- Waiting for bitcoind mining..."
mine-first-block
echo -n "- sending to address"
wait-for-node bitcoindnode "bitcoin-cli -regtest -rpcwait -rpcport=$rpcport  -rpcuser=$rpcuser -rpcpassword=$rpcpassword sendtoaddress $address 5"
mine-block
echo -n "- sending to address"
wait-for-node bitcoindnode "bitcoin-cli -regtest -rpcwait -rpcport=$rpcport  -rpcuser=$rpcuser -rpcpassword=$rpcpassword sendtoaddress $address 3"
mine-block
echo -n "- sending to address"
wait-for-node bitcoindnode "bitcoin-cli -regtest -rpcwait -rpcport=$rpcport  -rpcuser=$rpcuser -rpcpassword=$rpcpassword sendtoaddress $address 2"
mine-block



