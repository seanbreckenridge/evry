# reset a job (remove the tag file) for an evry job
job-reset() {
	local data_dir tags
	if [[ -z "$1" ]] || [[ -n "$JOB_RESET_USE_FZF" ]]; then
		data_dir="$(evry location - 2>/dev/null)"
		cd "${data_dir}" || return $?
		if tags="$(fzf -q "$*" -m)"; then
			echo -e "$tags" | while read -r tag; do
				command rm -v "${tag}"
			done
		else
			# user didn't select something with fzf, cd back to dir and fail
			cd - >/dev/null || return $?
			return 1
		fi
		cd - || return
	else
		tag="$(evry location -"$1")"
		if [[ -f "${tag}" ]]; then
			command rm -v "${tag}"
		else
			echo "No such tag file: ${tag}" >&2
			JOB_RESET_USE_FZF=1 job-reset "$1" || return $?
		fi
	fi
}

# pipe EVRY_JSON=1 json to this to describe when items run next
describe-evry-json() {
	jq -r '.[] | select((.type == "tag_name") or .type == "till_next_pretty") | .body' | paste -d " " - - | sed -e 's/ / - /'
}
