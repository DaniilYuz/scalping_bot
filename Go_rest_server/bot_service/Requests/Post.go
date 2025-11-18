package requests

import (
	"net/http"

	"github.com/gin-gonic/gin"
)

func AddRequest(context *gin.Context) {
	var newRequest RequestData
	if err := context.BindJSON(&newRequest); err != nil {
		context.IndentedJSON(http.StatusBadRequest, gin.H{"message": "invalid request"})
		return
	}

	Requests = append(Requests, newRequest)
	context.IndentedJSON(http.StatusCreated, newRequest)
}