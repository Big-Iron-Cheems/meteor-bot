package config

import (
	"github.com/joho/godotenv"
	"github.com/rs/zerolog/log"
	"os"
)

// GlobalConfig holds the environment variables required for the bot to run
var GlobalConfig Env

type Env struct {
	DiscordToken    string // Discord bot token
	ApiBase         string // Base URL for the API
	BackendToken    string // Backend token
	GuildId         string // ID of the guild where the bot is running
	CopeNnId        string // ID of the CopeNN emoji
	MemberCountId   string // ID of the member count channel
	DownloadCountId string // ID of the download count channel
	UptimeUrl       string // URL for the uptime monitor
	EnableLogFile   bool   // Enable logging to a file
}

func init() {
	if err := godotenv.Load(); err != nil {
		log.Warn().Err(err).Msg("Error loading .env file")
	}

	GlobalConfig = Env{
		DiscordToken:    os.Getenv("DISCORD_TOKEN"),
		ApiBase:         os.Getenv("API_BASE"),
		BackendToken:    os.Getenv("BACKEND_TOKEN"),
		GuildId:         os.Getenv("GUILD_ID"),
		CopeNnId:        os.Getenv("COPE_NN_ID"),
		MemberCountId:   os.Getenv("MEMBER_COUNT_ID"),
		DownloadCountId: os.Getenv("DOWNLOAD_COUNT_ID"),
		UptimeUrl:       os.Getenv("UPTIME_URL"),
		EnableLogFile:   os.Getenv("ENABLE_LOG_FILE") == "true",
	}
}
