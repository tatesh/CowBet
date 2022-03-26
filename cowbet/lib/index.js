module.exports = ({ wallets, refs, config, client }) => ({
  getCount: () => client.query("counter", { get_count: {} }),
  increment: (signer = wallets.bombay) =>
    client.execute(signer, "counter", { increment: {} }),
});
