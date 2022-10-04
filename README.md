# redstone-connector-rust
Attempt to use redstone oracle with ether-rs.


anvil -f http://10.8.0.1:9650/ext/bc/C/rpc -p 33461


            tx.data =
              tx.data +
              (await self.getPriceData(contract.signer)) +
              self.getMarkerData();



protected async getPriceData(signer: Signer, asset?: string): Promise<string> {
const {priceData, liteSignature} = await this.apiConnector.getSignedPrice();

    let data = this.priceSigner.getLiteDataBytesString(priceData);
    
    data += priceData.symbols.length.toString(16).padStart(2, "0")
          + liteSignature.substr(2);
    return data;
}



https://api.redstone.finance/prices/?symbol=AVAX&provider=redstone&limit=1

async getSignedPrice(): Promise<SignedPriceDataType> {
return await redstone.oracle.get(
this.priceFeedOptions.dataSources!,
this.priceFeedOptions.asset);
}