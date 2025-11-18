package models

import "gorm.io/gorm"

type User struct {
	gorm.Model
	Name     string `gorm:"unique" json:"name"`
	Password string `json:"password"`    
}