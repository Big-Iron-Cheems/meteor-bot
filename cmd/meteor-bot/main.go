package main

import (
	"github.com/bwmarrin/discordgo"
	"github.com/rs/zerolog/log"
	"meteor-bot/internal/app/commands"
	"meteor-bot/internal/app/common"
	"meteor-bot/internal/app/config"
	"meteor-bot/internal/app/events"
	"os"
	"os/signal"
)

func main() {
	// Initialize the logger
	common.InitLogger()
	defer common.CloseLogger()

	// Initialize config
	config.Init()

	// Create a new Discord session
	s, err := discordgo.New("Bot " + config.GlobalConfig.DiscordToken)
	if err != nil {
		log.Panic().Err(err).Msg("Error creating Discord session")
	}
	// Enable the intents required for the bot
	s.Identify.Intents |= discordgo.IntentsGuildMessages | discordgo.IntentsGuildMembers

	// Initialize the events and register them to the Discord API
	// Must be done BEFORE opening the session to make discordgo.Ready handlers work
	events.Init(s)

	// Bot is ready, open the session
	if err = s.Open(); err != nil {
		log.Panic().Err(err).Msg("Error opening Discord session")
	}
	defer s.Close()

	// Initialize the commands and handlers, and register them to the Discord API
	// Must be done AFTER opening the session to add the commands to the API
	commands.Init(s)

	// Wait until the bot is stopped
	stop := make(chan os.Signal, 1)
	signal.Notify(stop, os.Interrupt)
	log.Info().Msg("Press Ctrl+C to exit")
	<-stop

	log.Info().Msg("Gracefully shutting down.")
}
