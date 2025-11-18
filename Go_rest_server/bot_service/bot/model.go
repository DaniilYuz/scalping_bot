package bot

type BotCommand string
type BotStatus string

type Bot struct {
}

const (
	CommandStart   BotCommand = "Start"
	CommandStop    BotCommand = "Stop"
	CommandRestart BotCommand = "Restart"
)

const (
	StatusStart BotStatus = "Running"
	StatusStop  BotStatus = "Stop"
	StatusError BotStatus = "Error"
)

// model for control.go
type ControlBot struct {
	Command BotCommand `json:"commandbot"`
	Status  BotStatus  `json:"statusbot"`
}

// model for info.go
type InfoBot struct {
	Status BotStatus `json:"statusbot"`
	Config ConfigBot `json:"configbot"`
}

// type for config.go
type ConfigBot struct {
	Coins   string `json:"coins"`
	Streams []int  `json:"streams"`
}