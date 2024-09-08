package help

import (
	"fmt"
	"github.com/bwmarrin/discordgo"
	"meteor-bot/internal/app/common"
)

type LogsCommand struct {
	common.BaseCommand
}

func NewLogsCommand() *LogsCommand {
	return &LogsCommand{
		BaseCommand: *common.NewCommandBuilder().
			SetName("logs").
			SetDescription("Tells someone how to find the Minecraft logs").
			Build(),
	}
}

func (c *LogsCommand) Build() *discordgo.ApplicationCommand {
	return &discordgo.ApplicationCommand{
		Name:        c.Name(),
		Description: c.Description(),
		Options: []*discordgo.ApplicationCommandOption{
			{
				Name:        "member",
				Description: "The member to tell how to find the Minecraft logs",
				Type:        discordgo.ApplicationCommandOptionUser,
				Required:    true,
			},
		},
	}
}

func (c *LogsCommand) Handle(s *discordgo.Session, i *discordgo.InteractionCreate) {
	targetMember := i.ApplicationCommandData().Options[0].UserValue(s)
	c.HandleInteractionRespond(s, i, &discordgo.InteractionResponse{
		Type: discordgo.InteractionResponseChannelMessageWithSource,
		Data: &discordgo.InteractionResponseData{
			Embeds: []*discordgo.MessageEmbed{
				{
					Title:       "Find the Minecraft Logs",
					Description: fmt.Sprintf("%s The logs guide explains how to find and share your Minecraft logs, please read it.", targetMember.Mention()),
					Color:       common.EmbedColor,
				},
			},
			Components: []discordgo.MessageComponent{
				discordgo.ActionsRow{
					Components: []discordgo.MessageComponent{
						discordgo.Button{
							Label: "Guide",
							Style: discordgo.LinkButton,
							URL:   "https://meteorclient.com/faq/getting-log",
						},
					},
				},
			},
		},
	})
}
