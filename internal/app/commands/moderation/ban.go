package moderation

import (
	"fmt"
	"github.com/bwmarrin/discordgo"
	"meteor-bot/internal/app/common"
)

type BanCommand struct {
	common.BaseCommand
}

func NewBanCommand() *BanCommand {
	return &BanCommand{
		BaseCommand: *common.NewCommandBuilder().
			SetName("ban").
			SetDescription("Bans a member").
			Build(),
	}
}

func (c *BanCommand) Build() *discordgo.ApplicationCommand {
	return &discordgo.ApplicationCommand{
		Name:                     c.Name(),
		Description:              c.Description(),
		DefaultMemberPermissions: &common.BanMemberPermission,
		Options: []*discordgo.ApplicationCommandOption{
			{
				Name:        "member",
				Description: "The member to ban",
				Type:        discordgo.ApplicationCommandOptionUser,
				Required:    true,
			},
		},
	}
}

func (c *BanCommand) Handler() common.CommandHandler {
	return func(s *discordgo.Session, i *discordgo.InteractionCreate) {
		if i.Member.Permissions&common.BanMemberPermission != common.BanMemberPermission {
			c.HandleInteractionRespond(s, i, &discordgo.InteractionResponse{
				Type: discordgo.InteractionResponseChannelMessageWithSource,
				Data: &discordgo.InteractionResponseData{
					Content: "You do not have the required permissions to ban members.",
					Flags:   discordgo.MessageFlagsEphemeral,
				},
			})
			return
		}

		targetMember := i.ApplicationCommandData().Options[0].UserValue(s)
		if err := s.GuildBanCreate(i.GuildID, targetMember.ID, 0); err != nil {
			c.HandleInteractionRespond(s, i, &discordgo.InteractionResponse{
				Type: discordgo.InteractionResponseChannelMessageWithSource,
				Data: &discordgo.InteractionResponseData{
					Content: "An error occurred while banning the member.",
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
						Title:       "Member Banned",
						Description: fmt.Sprintf("Banned %s.", targetMember.Mention()),
						Color:       common.EmbedColor,
					},
				},
			},
		})
	}
}
