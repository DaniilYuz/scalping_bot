package requests

type RequestData struct {
	Id          string   `json:"id"`
	FirstCoin   string   `json:"firstCoin"`
	SecondCoin  string   `json:"secondCoin"`
	StreamTypes []string `json:"streamTypes"`
	Completed   bool     `json:"completed"`
}

var Requests = []RequestData{
	{Id: "1", FirstCoin: "btc", SecondCoin: "usdt", StreamTypes: []string{"kline"}},
	{Id: "2", FirstCoin: "eth", SecondCoin: "usdt", StreamTypes: []string{"aggTrade"}},
	{Id: "3", FirstCoin: "btc", SecondCoin: "usdt", StreamTypes: []string{"DepthOrderBook"}},
}