package moderation

import (
	"errors"
	"fmt"
	"github.com/bwmarrin/discordgo"
	"meteor-bot/internal/app/common"
	"regexp"
	"strconv"
	"time"
)

type MuteCommand struct {
	common.BaseCommand
}

func NewMuteCommand() *MuteCommand {
	return &MuteCommand{
		BaseCommand: *common.NewCommandBuilder().
			SetName("mute").
			SetDescription("Mutes a member").
			Build(),
	}
}

func (c *MuteCommand) Build() *discordgo.ApplicationCommand {
	return &discordgo.ApplicationCommand{
		Name:                     c.Name(),
		Description:              c.Description(),
		DefaultMemberPermissions: &common.ModerateMembersPermission,
		Options: []*discordgo.ApplicationCommandOption{
			{
				Name:        "member",
				Description: "The member to mute",
				Type:        discordgo.ApplicationCommandOptionUser,
				Required:    true,
			},
			{
				Name:        "duration",
				Description: "The duration of the mute",
				Type:        discordgo.ApplicationCommandOptionString,
				Required:    true,
			},
			{
				Name:        "reason",
				Description: "The reason for the mute",
				Type:        discordgo.ApplicationCommandOptionString,
				Required:    false,
			},
		},
	}
}

func (c *MuteCommand) Handle(s *discordgo.Session, i *discordgo.InteractionCreate) {
	if i.Member.Permissions&common.ModerateMembersPermission == 0 {
		c.HandleInteractionRespond(s, i, &discordgo.InteractionResponse{
			Type: discordgo.InteractionResponseChannelMessageWithSource,
			Data: &discordgo.InteractionResponseData{
				Content: "You do not have the required permissions to mute members.",
				Flags:   discordgo.MessageFlagsEphemeral,
			},
		})
		return
	}

	targetMember := i.ApplicationCommandData().Options[0].UserValue(s)
	durationStr := i.ApplicationCommandData().Options[1].StringValue()
	duration, err := parseDuration(durationStr)
	if err != nil {
		c.HandleInteractionRespond(s, i, &discordgo.InteractionResponse{
			Type: discordgo.InteractionResponseChannelMessageWithSource,
			Data: &discordgo.InteractionResponseData{
				Content: "Invalid duration format. Please use the format `1s`, `1m`, `1h`, `1d`, or `1w`.",
				Flags:   discordgo.MessageFlagsEphemeral,
			},
		})
		return
	}

	// Fetch optional arguments
	reason := "Reason unspecified"
	for _, opt := range i.ApplicationCommandData().Options {
		if opt.Name == "reason" {
			reason = opt.StringValue()
			break
		}
	}

	targetGuildMember, ok := i.ApplicationCommandData().Resolved.Members[targetMember.ID]
	if !ok {
		c.HandleInteractionRespond(s, i, &discordgo.InteractionResponse{
			Type: discordgo.InteractionResponseChannelMessageWithSource,
			Data: &discordgo.InteractionResponseData{
				Content: "An error occurred while fetching the member.",
				Flags:   discordgo.MessageFlagsEphemeral,
			},
		})
		return
	}

	// Check if the member is already muted
	if targetGuildMember.CommunicationDisabledUntil != nil && targetGuildMember.CommunicationDisabledUntil.After(time.Now()) {
		c.HandleInteractionRespond(s, i, &discordgo.InteractionResponse{
			Type: discordgo.InteractionResponseChannelMessageWithSource,
			Data: &discordgo.InteractionResponseData{
				Content: "Member is already muted.",
				Flags:   discordgo.MessageFlagsEphemeral,
			},
		})
		return
	}

	// Check if the target member cannot be muted
	if targetGuildMember.Permissions&common.ModerateMembersPermission != 0 {
		c.HandleInteractionRespond(s, i, &discordgo.InteractionResponse{
			Type: discordgo.InteractionResponseChannelMessageWithSource,
			Data: &discordgo.InteractionResponseData{
				Content: "You do not have the required permissions to mute this member.",
				Flags:   discordgo.MessageFlagsEphemeral,
			},
		})
		return
	}

	// Mute the member
	muteUntil := time.Now().Add(duration)
	err = s.GuildMemberTimeout(i.GuildID, targetMember.ID, &muteUntil, discordgo.WithAuditLogReason(reason))
	if err != nil {
		c.HandleInteractionRespond(s, i, &discordgo.InteractionResponse{
			Type: discordgo.InteractionResponseChannelMessageWithSource,
			Data: &discordgo.InteractionResponseData{
				Content: "An error occurred while muting the member.",
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
					Title:       "Member Muted",
					Description: fmt.Sprintf("Muted %s for %s.", targetMember.Mention(), durationStr),
					Color:       common.EmbedColor,
				},
			},
		},
	})

}

// parseDuration parses the duration string and return the time.Duration value
func parseDuration(durationStr string) (time.Duration, error) {
	re := regexp.MustCompile(`^(\d+)([smhdw])$`)
	matches := re.FindStringSubmatch(durationStr)
	if matches == nil {
		return 0, errors.New("invalid duration format")
	}

	value, err := strconv.Atoi(matches[1])
	if err != nil || value <= 0 {
		return 0, errors.New("invalid duration value")
	}

	unit := matches[2]
	var duration time.Duration
	switch unit {
	case "s":
		duration = time.Duration(value) * time.Second
	case "m":
		duration = time.Duration(value) * time.Minute
	case "h":
		duration = time.Duration(value) * time.Hour
	case "d":
		duration = time.Duration(value) * 24 * time.Hour
	case "w":
		duration = time.Duration(value) * 7 * 24 * time.Hour
	default:
		return 0, errors.New("invalid duration unit")
	}

	// Check if the duration is within the allowed range
	if duration > 2419200*time.Second {
		return 0, errors.New("duration exceeds the maximum allowed value of 4 weeks")
	}

	return duration, nil
}
