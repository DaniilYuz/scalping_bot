package bot

import (
	"net/http"

	"github.com/gin-gonic/gin"
)

func InfoHandler(context *gin.Context){
	currentInfo := InfoBot{
		Status: currentBot.Status,
		Config: currentConfig,
	}

	context.IndentedJSON(http.StatusOK, currentInfo)
}