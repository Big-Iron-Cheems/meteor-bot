package config

import (
	"errors"
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
	ApplicationId   string // ID of the application
	GuildId         string // ID of the guild where the bot is running
	CopeNnId        string // ID of the CopeNN emoji
	MemberCountId   string // ID of the member count channel
	DownloadCountId string // ID of the download count channel
	UptimeUrl       string // URL for the uptime monitor
	EnableLogFile   bool   // Enable logging to a file
}

func Init() {
	if _, err := os.Stat(".env"); errors.Is(err, os.ErrNotExist) {
		log.Warn().Msg(".env file not found, using current environment")
	} else {
		if err := godotenv.Load(); err != nil {
			log.Panic().Err(err).Msg("Error loading .env file")
		}
	}

	GlobalConfig = Env{
		DiscordToken:    os.Getenv("DISCORD_TOKEN"),
		ApiBase:         os.Getenv("API_BASE"),
		BackendToken:    os.Getenv("BACKEND_TOKEN"),
		ApplicationId:   os.Getenv("APPLICATION_ID"),
		GuildId:         os.Getenv("GUILD_ID"),
		CopeNnId:        os.Getenv("COPE_NN_ID"),
		MemberCountId:   os.Getenv("MEMBER_COUNT_ID"),
		DownloadCountId: os.Getenv("DOWNLOAD_COUNT_ID"),
		UptimeUrl:       os.Getenv("UPTIME_URL"),
		EnableLogFile:   os.Getenv("ENABLE_LOG_FILE") == "true",
	}

	log.Info().Msg("Config initialized")
}
