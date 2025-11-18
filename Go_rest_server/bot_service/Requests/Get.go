package requests

import (
	"errors"
	"net/http"

	"github.com/gin-gonic/gin"
)

func GetRequests(context *gin.Context) {
	context.IndentedJSON(http.StatusOK, Requests)
}

func FindRequestById(id string) (*RequestData, error) {
	for i, t := range Requests {
		if t.Id == id {
			return &Requests[i], nil
		}
	}
	return nil, errors.New("request not found")
}

func GetRequestById(context *gin.Context) {
	id := context.Param("Id")
	request, err := FindRequestById(id)
	if err != nil {
		context.IndentedJSON(http.StatusNotFound, gin.H{"message": "request not found"})
		return
	}
	context.IndentedJSON(http.StatusOK, request)
}