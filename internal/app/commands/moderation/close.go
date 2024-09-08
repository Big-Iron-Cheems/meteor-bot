package moderation

import (
	"github.com/bwmarrin/discordgo"
	"meteor-bot/internal/app/common"
)

type CloseCommand struct {
	common.BaseCommand
}

func NewCloseCommand() *CloseCommand {
	return &CloseCommand{
		BaseCommand: *common.NewCommandBuilder().
			SetName("close").
			SetDescription("Locks the current forum post").
			Build(),
	}
}

func (c *CloseCommand) Build() *discordgo.ApplicationCommand {
	return &discordgo.ApplicationCommand{
		Name:                     c.Name(),
		Description:              c.Description(),
		DefaultMemberPermissions: &common.ManageThreadsPermission,
	}
}

func (c *CloseCommand) Handle(s *discordgo.Session, i *discordgo.InteractionCreate) {
	if i.Member.Permissions&common.ManageThreadsPermission != common.ManageThreadsPermission {
		c.HandleInteractionRespond(s, i, &discordgo.InteractionResponse{
			Type: discordgo.InteractionResponseChannelMessageWithSource,
			Data: &discordgo.InteractionResponseData{
				Content: "You do not have the required permissions to close threads.",
				Flags:   discordgo.MessageFlagsEphemeral,
			},
		})
		return
	}

	// Check if the command was used in a forum channel
	channel, err := s.State.Channel(i.ChannelID)
	if err != nil {
		c.HandleInteractionRespond(s, i, &discordgo.InteractionResponse{
			Type: discordgo.InteractionResponseChannelMessageWithSource,
			Data: &discordgo.InteractionResponseData{
				Content: "An error occurred while fetching the channel.",
			},
		})
		return
	}
	if !channel.IsThread() {
		c.HandleInteractionRespond(s, i, &discordgo.InteractionResponse{
			Type: discordgo.InteractionResponseChannelMessageWithSource,
			Data: &discordgo.InteractionResponseData{
				Content: "This command can only be used in forum channels.",
				Flags:   discordgo.MessageFlagsEphemeral,
			},
		})
		return
	}

	// Respond to the interaction
	c.HandleInteractionRespond(s, i, &discordgo.InteractionResponse{
		Type: discordgo.InteractionResponseChannelMessageWithSource,
		Data: &discordgo.InteractionResponseData{
			Embeds: []*discordgo.MessageEmbed{
				{
					Title:       "Thread Closed",
					Description: "This thread is now locked.",
					Color:       common.EmbedColor,
				},
			},
		},
	})

	// Close the thread
	locked := true
	archived := true
	_, err = s.ChannelEditComplex(i.ChannelID, &discordgo.ChannelEdit{
		Locked:   &locked,
		Archived: &archived,
	})
	if err != nil {
		c.HandleInteractionRespond(s, i, &discordgo.InteractionResponse{
			Type: discordgo.InteractionResponseChannelMessageWithSource,
			Data: &discordgo.InteractionResponseData{
				Content: "An error occurred while closing the thread.",
				Flags:   discordgo.MessageFlagsEphemeral,
			},
		})
		return
	}
}
