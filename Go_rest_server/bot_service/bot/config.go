package bot

import (
	"net/http"

	"github.com/gin-gonic/gin"
)
	

var currentConfig ConfigBot = ConfigBot{
	Coins: "",
	Streams: []int{0},
}

func SetConfig(context *gin.Context){
	var newConfig ConfigBot

	if err:= context.BindJSON(&newConfig); err != nil{
		context.IndentedJSON(http.StatusBadRequest, gin.H{"error" : "invalid config"})
		return
	}
	currentConfig = newConfig 
	context.IndentedJSON(http.StatusCreated, currentConfig)
}

func EditConfig(context *gin.Context){
	var partialConfig ConfigBot

	if err := context.BindJSON(&partialConfig); err != nil {
		context.IndentedJSON(http.StatusBadRequest, gin.H{"error": "invalid config"})
		return
	}

	if len(partialConfig.Coins) > 0 {
		if partialConfig.Coins != ""{
		currentConfig.Coins = partialConfig.Coins
		}
	}
	
	if len(partialConfig.Streams) > 0 {
		currentConfig.Streams = partialConfig.Streams
	}

	context.IndentedJSON(http.StatusOK, currentConfig)
}