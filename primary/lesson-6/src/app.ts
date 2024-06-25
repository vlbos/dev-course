import { ApiPromise, WsProvider, Keyring } from "@polkadot/api";
import { Header } from "@polkadot/types/interfaces/runtime/types";

const local_url = "ws://127.0.0.1:9944";
const remote = "wss://polkadot.api.onfinality.io/public-ws";

// sleep for a while for subscribe method.
function delay(ms: number) {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

const main = async () => {
  // ---------------------------------  header
  const wsProvider = new WsProvider(local_url);
  const api: ApiPromise = await ApiPromise.create({
    provider: wsProvider,
    types: {},
  });
  await api.isReady;

  // ---------------------------------  const
  // The amount required to create a new account
  console.log(api.consts.balances.existentialDeposit.toHuman());
  console.log(api.consts.timestamp.minimumPeriod.toHuman());

  // --------------------------------- variable
  console.log((await api.query.timestamp.now()).toHuman());

  const ADDR1 = "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY";
  const ADDR2 = "5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty";

  const account = await api.query.system.account(ADDR1);
  console.log("alice account: ", account.toHuman());

  // --------------------------------- iterate map data
  const entries = await api.query.system.account.entries();
  entries.forEach(([key, value]) => {
    console.log(key.toHuman());
    console.log(value.toHuman());
  });

  // ---------------------------------  rpc get data
  // get some data not just a on-chain storage
  const chain = await api.rpc.system.chain();
  console.log(chain.toHuman());

  // Retrieve the latest header
  const lastHeader = await api.rpc.chain.getHeader();
  console.log(lastHeader.toHuman());

  const metadata = await api.rpc.state.getMetadata();
  console.log("++++++");
  console.log(metadata.toHuman());
  // console.log(metadata.toString());

  // ---------------------------------  multi get data

  const balances = await api.query.system.account.multi([ADDR1, ADDR2]);

  const data1 = balances[0];
  const data2 = balances[0];
  console.log(data1.toHuman(), data2.toHuman());

  const results = await api.queryMulti([
    api.query.timestamp.now,
    [api.query.system.account, ADDR1],
  ]);
  const timestamp = results[0];
  const account2 = results[1];
  console.log(timestamp.toHuman(), account2.toHuman());

  // ---------------------------------  keyring
  const keyring = new Keyring({ type: "sr25519" });
  const alice = keyring.addFromUri("//Alice");
  const bob = keyring.addFromUri("//Bob");
  console.log(alice.address, bob.address);

  // ---------------------------------  extrinsic
  const tx = api.tx.system.remark("0x1234");
  const hash = await tx.signAndSend(alice);
  console.log(`Add stake transaction sent with hash ${hash.toHex()}`);

  const tx2 = api.tx.balances.transferKeepAlive(bob.address, 12345);
  const hash2 = await tx2.signAndSend(alice);
  console.log(`Add stake transaction sent with hash ${hash2.toHex()}`);

  const tx3 = api.tx.templateModule.doSomething(12345);
  const hash3 = await tx3.signAndSend(alice);
  console.log(`Add stake transaction sent with hash ${hash3.toHex()}`);

  // ---------------------------------  subscribe
  // single data
  await api.rpc.chain.subscribeNewHeads((header) => {
    console.log(`Chain is at block: ${header}`);
  });

  // with parameter
  await api.query.system.account(
    ADDR1,
    (account: {
      nonce: number;
      data: {
        free: string;
      };
    }) => {
      console.log(
        `Current nonce is ${account.nonce}, balance is ${account.data.free}`,
      );
    },
  );

  // customer query
  await api.query.templateModule.something((data: number) => {
    console.log(`Chain data: ${data}`);
  });

  // ---------------------------------  runtime call
  console.log((await api.call.accountNonceApi.accountNonce(ADDR1)).toHuman());
  console.log((await api.call.auraApi.authorities()).toHuman());

  // ---------------------------------  type usage
  const lastHeader2: Header = await api.rpc.chain.getHeader();
  console.log(lastHeader2.number.toHuman());

  // await delay(20000);
  console.log("Hello");
};

main()
  .then(() => {
    console.log("successfully exited");
    process.exit(0);
  })
  .catch((err) => {
    console.log("error occur:", err);
    process.exit(1);
  });
