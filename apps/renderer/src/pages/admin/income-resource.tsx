import {
	Create,
	DateField,
	DateInput,
	Datagrid,
	Edit,
	List,
	NumberField,
	NumberInput,
	SimpleForm,
	TextField,
	TextInput,
} from "react-admin"

export function IncomeList() {
	return (
		<List>
			<Datagrid rowClick="edit">
				<DateField source="date" />
				<TextField source="description" />
				<TextField source="category" />
				<NumberField source="amount" />
			</Datagrid>
		</List>
	)
}

export function IncomeEdit() {
	return (
		<Edit>
			<SimpleForm>
				<DateInput source="date" />
				<TextInput source="description" />
				<TextInput source="category" />
				<NumberInput source="amount" />
			</SimpleForm>
		</Edit>
	)
}

export function IncomeCreate() {
	return (
		<Create>
			<SimpleForm>
				<DateInput source="date" />
				<TextInput source="description" />
				<TextInput source="category" />
				<NumberInput source="amount" />
			</SimpleForm>
		</Create>
	)
}
