package help

import (
	"fmt"
	"github.com/bwmarrin/discordgo"
	"meteor-bot/internal/app/common"
)

type OldVersionCommand struct {
	common.BaseCommand
}

func NewOldVersionCommand() *OldVersionCommand {
	return &OldVersionCommand{
		BaseCommand: *common.NewCommandBuilder().
			SetName("old-versions").
			SetDescription("Tells someone how to play on older versions of Minecraft").
			Build(),
	}
}

func (c *OldVersionCommand) Build() *discordgo.ApplicationCommand {
	return &discordgo.ApplicationCommand{
		Name:        c.Name(),
		Description: c.Description(),
		Options: []*discordgo.ApplicationCommandOption{
			{
				Name:        "member",
				Description: "The member to tell how to play on older versions of Minecraft",
				Type:        discordgo.ApplicationCommandOptionUser,
				Required:    true,
			},
		},
	}
}

func (c *OldVersionCommand) Handle(s *discordgo.Session, i *discordgo.InteractionCreate) {
	targetMember := i.ApplicationCommandData().Options[0].UserValue(s)
	c.HandleInteractionRespond(s, i, &discordgo.InteractionResponse{
		Type: discordgo.InteractionResponseChannelMessageWithSource,
		Data: &discordgo.InteractionResponseData{
			Embeds: []*discordgo.MessageEmbed{
				{
					Title:       "Old Versions Guide",
					Description: fmt.Sprintf("%s The old version guide explains how to play on older versions of Minecraft, please read it.", targetMember.Mention()),
					Color:       common.EmbedColor,
				},
			},
			Components: []discordgo.MessageComponent{
				discordgo.ActionsRow{
					Components: []discordgo.MessageComponent{
						discordgo.Button{
							Label: "Guide",
							Style: discordgo.LinkButton,
							URL:   "https://meteorclient.com/faq/old-versions",
						},
					},
				},
			},
		},
	})
}
