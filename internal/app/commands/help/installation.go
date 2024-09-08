package help

import (
	"fmt"
	"github.com/bwmarrin/discordgo"
	"meteor-bot/internal/app/common"
)

type InstallationCommand struct {
	common.BaseCommand
}

func NewInstallationCommand() *InstallationCommand {
	return &InstallationCommand{
		BaseCommand: *common.NewCommandBuilder().
			SetName("installation").
			SetDescription("Tells someone to read the installation guide").
			Build(),
	}
}

func (c *InstallationCommand) Build() *discordgo.ApplicationCommand {
	return &discordgo.ApplicationCommand{
		Name:        c.Name(),
		Description: c.Description(),
		Options: []*discordgo.ApplicationCommandOption{
			{
				Name:        "member",
				Description: "The member to tell to read the installation guide",
				Type:        discordgo.ApplicationCommandOptionUser,
				Required:    true,
			},
		},
	}
}

func (c *InstallationCommand) Handle(s *discordgo.Session, i *discordgo.InteractionCreate) {
	targetMember := i.ApplicationCommandData().Options[0].UserValue(s)
	c.HandleInteractionRespond(s, i, &discordgo.InteractionResponse{
		Type: discordgo.InteractionResponseChannelMessageWithSource,
		Data: &discordgo.InteractionResponseData{
			Embeds: []*discordgo.MessageEmbed{
				{
					Title:       "Read the Installation Guide",
					Description: fmt.Sprintf("%s The installation guide answers your question, please read it.", targetMember.Mention()),
					Color:       common.EmbedColor,
				},
			},
			Components: []discordgo.MessageComponent{
				discordgo.ActionsRow{
					Components: []discordgo.MessageComponent{
						discordgo.Button{
							Label: "Guide",
							Style: discordgo.LinkButton,
							URL:   "https://meteorclient.com/faq/installation",
						},
					},
				},
			},
		},
	})
}
