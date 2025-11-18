package requests

import (
	"net/http"

	"github.com/gin-gonic/gin"
)

func ToggleRequestStatus(context *gin.Context) {
	id := context.Param("Id")
	request, err := FindRequestById(id)
	if err != nil {
		context.IndentedJSON(http.StatusNotFound, gin.H{"message": "request not found"})
		return
	}

	request.Completed = !request.Completed
	context.IndentedJSON(http.StatusOK, request)
}