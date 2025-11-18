package initializers

import "auth_service/models"

func SyncDatabase() {
	DB.AutoMigrate(&models.User{})
}