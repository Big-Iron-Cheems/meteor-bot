package commands

import (
	"github.com/bwmarrin/discordgo"
	"github.com/rs/zerolog/log"
	"meteor-bot/internal/app/commands/help"
	"meteor-bot/internal/app/commands/moderation"
	"meteor-bot/internal/app/commands/silly"
	"meteor-bot/internal/app/common"
)

var (
	commands        []*discordgo.ApplicationCommand
	commandHandlers map[string]func(s *discordgo.Session, i *discordgo.InteractionCreate)

	registeredCommands []*discordgo.ApplicationCommand
)

func Init(s *discordgo.Session) {
	// Initialize the commands and handlers map
	if commandHandlers == nil {
		commandHandlers = make(map[string]func(s *discordgo.Session, i *discordgo.InteractionCreate))
	}

	// Initialize the commands and handlers
	initCommands(
		moderation.NewBanCommand(),
		moderation.NewMuteCommand(),
		moderation.NewUnmuteCommand(),
		moderation.NewCloseCommand(),
		help.NewFaqCommand(),
		help.NewInstallationCommand(),
		help.NewLogsCommand(),
		help.NewOldVersionCommand(),
		silly.NewCapyCommand(),
		silly.NewCatCommand(),
		silly.NewDogCommand(),
		silly.NewMonkeyCommand(),
		silly.NewPandaCommand(),
		NewLinkCommand(),
		NewStatsCommand(),
	)

	// Add the handlers to the Discord session
	s.AddHandler(func(s *discordgo.Session, i *discordgo.InteractionCreate) {
		if h, ok := commandHandlers[i.ApplicationCommandData().Name]; ok {
			h(s, i)
		}
	})

	// Register the commands to the Discord API
	registerCommands(s)
}

func initCommands(cmds ...common.Command) {
	for _, cmd := range cmds {
		commands = append(commands, cmd.Build())
		commandHandlers[cmd.Name()] = cmd.Handle
	}
}

// registerCommands registers the commands to the Discord API
func registerCommands(s *discordgo.Session) {
	var err error
	registeredCommands, err = s.ApplicationCommandBulkOverwrite(s.State.User.ID, "", commands)
	if err != nil {
		log.Panic().Err(err).Msg("Cannot register commands")
	}

	log.Info().Msgf("%d commands registered successfully", len(registeredCommands))
}

// RemoveCommands removes the commands from the Discord API
func RemoveCommands(s *discordgo.Session) {
	log.Info().Msgf("Removing %d commands...", len(registeredCommands))
	_, err := s.ApplicationCommandBulkOverwrite(s.State.User.ID, "", nil)
	if err != nil {
		log.Panic().Err(err).Msg("Cannot remove commands")
	}
}
