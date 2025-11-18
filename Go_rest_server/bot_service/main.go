package main

import (
	"bot_service/bot"
	"os"

	"github.com/gin-gonic/gin"
)

func main() {
	router := gin.Default()

	botRoutes := router.Group("/bot")
	{
		botRoutes.GET("/info", bot.InfoHandler)
		botRoutes.POST("/config", bot.SetConfig)
		botRoutes.PATCH("/config", bot.EditConfig)
		botRoutes.POST("/control", bot.ControlHandler)
	}

	port := os.Getenv("PORT")
	if port == "" {
		port = "9091"
	}

	router.Run(":" + port)
}