package help

import (
	"fmt"
	"github.com/bwmarrin/discordgo"
	"meteor-bot/internal/app/common"
)

type FaqCommand struct {
	common.BaseCommand
}

func NewFaqCommand() *FaqCommand {
	return &FaqCommand{
		BaseCommand: *common.NewCommandBuilder().
			SetName("faq").
			SetDescription("Tells someone to read the FAQ").
			Build(),
	}
}

func (c *FaqCommand) Build() *discordgo.ApplicationCommand {
	return &discordgo.ApplicationCommand{
		Name:        c.Name(),
		Description: c.Description(),
		Options: []*discordgo.ApplicationCommandOption{
			{
				Name:        "member",
				Description: "The member to tell to read the FAQ",
				Type:        discordgo.ApplicationCommandOptionUser,
				Required:    true,
			},
		},
	}
}

func (c *FaqCommand) Handler() common.CommandHandler {
	return func(s *discordgo.Session, i *discordgo.InteractionCreate) {
		targetMember := i.ApplicationCommandData().Options[0].UserValue(s)
		c.HandleInteractionRespond(s, i, &discordgo.InteractionResponse{
			Type: discordgo.InteractionResponseChannelMessageWithSource,
			Data: &discordgo.InteractionResponseData{
				Embeds: []*discordgo.MessageEmbed{
					{
						Title:       "Read the FAQ",
						Description: fmt.Sprintf("%s The FAQ answers your question, please read it.", targetMember.Mention()),
						Color:       common.EmbedColor,
					},
				},
				Components: []discordgo.MessageComponent{
					discordgo.ActionsRow{
						Components: []discordgo.MessageComponent{
							discordgo.Button{
								Label: "FAQ",
								Style: discordgo.LinkButton,
								URL:   "https://meteorclient.com/faq",
							},
						},
					},
				},
			},
		})
	}
}
