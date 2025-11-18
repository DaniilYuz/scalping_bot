package bot

import (
	"github.com/gin-gonic/gin"
)

var currentBot ControlBot = ControlBot{
	Command: CommandStop,
	Status: StatusStop,
}

// POST /bot/control
func ControlHandler(context *gin.Context) {
   
}