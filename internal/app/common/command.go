package common

import (
	dg "github.com/bwmarrin/discordgo"
	"github.com/rs/zerolog/log"
)

var (
	ModerateMembersPermission int64 = dg.PermissionModerateMembers
	BanMemberPermission       int64 = dg.PermissionBanMembers
	ManageThreadsPermission   int64 = dg.PermissionManageThreads

	EmbedColor int = 0x913de2
)

// CommandHandler type alias for the Command.Handler function
type CommandHandler func(s *dg.Session, i *dg.InteractionCreate)

// Command interface for all slash commands
type Command interface {
	Name() string                  // Returns the name of the command
	Description() string           // Returns the description of the command
	Build() *dg.ApplicationCommand // Builds the command
	Handler() CommandHandler       // Func called when command is triggered
	HandleInteractionRespond(s *dg.Session, i *dg.InteractionCreate, resp *dg.InteractionResponse)
}

/*
BaseCommand struct that all commands should embed
This provides default implementations for the Command interface
*/
type BaseCommand struct {
	name        string
	description string
}

func (c *BaseCommand) Name() string {
	return c.name
}

func (c *BaseCommand) Description() string {
	return c.description
}

/*
HandleInteractionRespond responds with the given response and logs the error if there is one
Wraps the discordgo.Session#InteractionRespond function
*/
func (c *BaseCommand) HandleInteractionRespond(s *dg.Session, i *dg.InteractionCreate, resp *dg.InteractionResponse) {
	if err := s.InteractionRespond(i.Interaction, resp); err != nil {
		log.Error().Err(err).Msg("Error responding to interaction")
	}
}
