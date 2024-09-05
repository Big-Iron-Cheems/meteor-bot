package moderation

import (
	"fmt"
	"github.com/bwmarrin/discordgo"
	"meteor-bot/internal/app/common"
	"time"
)

type UnmuteCommand struct {
	common.BaseCommand
}

func NewUnmuteCommand() *UnmuteCommand {
	return &UnmuteCommand{
		BaseCommand: *common.NewCommandBuilder().
			SetName("unmute").
			SetDescription("Unmutes a member").
			Build(),
	}
}

func (c *UnmuteCommand) Build() *discordgo.ApplicationCommand {
	return &discordgo.ApplicationCommand{
		Name:                     c.Name(),
		Description:              c.Description(),
		DefaultMemberPermissions: &common.ModerateMembersPermission,
		Options: []*discordgo.ApplicationCommandOption{
			{
				Name:        "member",
				Description: "The member to unmute",
				Type:        discordgo.ApplicationCommandOptionUser,
				Required:    true,
			},
		},
	}
}

func (c *UnmuteCommand) Handle(s *discordgo.Session, i *discordgo.InteractionCreate) {
	if i.Member.Permissions&common.ModerateMembersPermission != common.ModerateMembersPermission {
		c.HandleInteractionRespond(s, i, &discordgo.InteractionResponse{
			Type: discordgo.InteractionResponseChannelMessageWithSource,
			Data: &discordgo.InteractionResponseData{
				Content: "You do not have the required permissions to unmute members.",
				Flags:   discordgo.MessageFlagsEphemeral,
			},
		})
		return
	}

	targetMember := i.ApplicationCommandData().Options[0].UserValue(s)
	targetGuildMember, err := s.GuildMember(targetMember.ID, i.GuildID)
	if err != nil {
		c.HandleInteractionRespond(s, i, &discordgo.InteractionResponse{
			Type: discordgo.InteractionResponseChannelMessageWithSource,
			Data: &discordgo.InteractionResponseData{
				Content: "An error occurred while fetching the member.",
				Flags:   discordgo.MessageFlagsEphemeral,
			},
		})
		return
	}

	// Check if the member is not muted, if so, return
	if targetGuildMember.CommunicationDisabledUntil == nil || targetGuildMember.CommunicationDisabledUntil.Before(time.Now()) {
		c.HandleInteractionRespond(s, i, &discordgo.InteractionResponse{
			Type: discordgo.InteractionResponseChannelMessageWithSource,
			Data: &discordgo.InteractionResponseData{
				Content: "Member is not muted.",
				Flags:   discordgo.MessageFlagsEphemeral,
			},
		})
		return
	}

	// Unmute the member
	err = s.GuildMemberTimeout(i.GuildID, targetMember.ID, nil)
	if err != nil {
		c.HandleInteractionRespond(s, i, &discordgo.InteractionResponse{
			Type: discordgo.InteractionResponseChannelMessageWithSource,
			Data: &discordgo.InteractionResponseData{
				Content: "An error occurred while unmuting the member.",
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
					Title:       "Member Unmuted",
					Description: fmt.Sprintf("Unmuted %s.", targetMember.Mention()),
					Color:       common.EmbedColor,
				},
			},
		},
	})
}
