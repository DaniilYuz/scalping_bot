package main

import (
	"auth_service/controllers"
	"auth_service/initializers"
	"auth_service/middleware"

	"github.com/gin-gonic/gin"
)

func init() {
	initializers.LoadEnvVariables()
	initializers.ConnectToDb()
	initializers.SyncDatabase()
}

func main() {
	r := gin.Default()

	// Auth routes
	r.POST("/signup", controllers.Signup)
	r.POST("/login", controllers.Login)
	
	// Protected routes
	protected := r.Group("/")
	protected.Use(middleware.RequireAuth)
	{
		protected.GET("/validate", controllers.Validate)
	}

	r.Run() // listens on :9090
}